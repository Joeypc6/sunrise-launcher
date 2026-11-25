//! `native-build`: building a profile's native mods, their catalogs and internals, or a server engine.
use super::*;

/// The `--profile` value. [`native_mods::Profile`] lives in the launcher's pure type crate, which
/// stays free of clap, so the command line parses its own one-variant name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub(super) enum ProfileArg {
    Optimized,
}

impl From<ProfileArg> for native_mods::Profile {
    fn from(_: ProfileArg) -> Self {
        Self::Optimized
    }
}

// `native-build`'s options, read by clap as they were from the variant's own fields.
#[derive(clap::Args)]
pub(super) struct NativeBuildArgs {
    /// Discover internal uses through compiler diagnostics. Prints resolved declarations
    /// as JSON for cargo sunrise-mod internals sync; never installs or selects a game
    #[arg(long, conflicts_with_all = ["server_output", "instance", "version", "profile"], requires = "folder")]
    pub(super) sync_internals: bool,
    #[arg(long, requires = "folder", conflicts_with_all = ["sync_internals", "server_output", "instance", "version", "profile"])]
    pub(super) catalog: bool,
    /// Copy discovery catalogs into this caller-owned empty directory
    #[arg(long, requires = "catalog")]
    pub(super) catalog_output: Option<PathBuf>,
    /// A mod folder (sunrise-mod.toml with [native], Cargo.toml and src/), or a folder of them
    /// (default: the native mods installed and turned on in the profile)
    pub(super) folder: Option<PathBuf>,
    /// Export mode: build a standalone dedicated-server engine into this folder instead of
    /// a profile's build. Needs a mod FOLDER; no profile is read or changed, and no
    /// launcher config is written
    #[arg(
        long = "server-output",
        value_name = "DIR",
        conflicts_with = "instance",
        requires = "folder"
    )]
    pub(super) server_output: Option<PathBuf>,
    /// Allow exact-hash jar downloads into the local cache for the server export, which
    /// copies the jars its mods list into its own jars folder (the operator's copy; nothing we
    /// publish contains one); jars of mods from the Sunrise registry download without this
    #[arg(long, requires = "server_output")]
    pub(super) allow_jar_downloads: bool,
    /// The profile whose installed native mods to build, and which then plays the build
    /// (default: the selected one)
    #[arg(long)]
    pub(super) instance: Option<String>,
    /// For a profile, assert its current base. In server-export mode, select the installed
    /// game build to use (defaults to the newest installed release or Nightly)
    #[arg(long)]
    pub(super) version: Option<String>,
    /// Build native code the registry did not review: a folder, or a profile's mods that were
    /// copied in by hand (asked again on every run)
    #[arg(long)]
    pub(super) trust_native_code: bool,
    /// Native mods use the pinned SDK's optimized compilation settings.
    /// The game is never rebuilt by a mod build.
    #[arg(long, value_enum)]
    pub(super) profile: Option<ProfileArg>,
    /// Build the horizon-mixins tool from its pinned source instead of downloading the
    /// prebuilt one: for CI, which keeps that path proven
    #[arg(long, hide = true)]
    pub(super) mixins_from_source: bool,
}

pub(super) fn build(args: NativeBuildArgs) -> Result<i32, Error> {
    let NativeBuildArgs {
        sync_internals,
        catalog,
        catalog_output,
        folder,
        server_output,
        allow_jar_downloads,
        instance,
        version,
        trust_native_code,
        profile,
        mixins_from_source,
    } = args;
    let profile = profile.map(native_mods::Profile::from);
    native_mods::mixins_from_source(mixins_from_source);
    if catalog {
        native_permission(trust_native_code)?;
        let destination = catalog_output.ok_or("--catalog requires --catalog-output")?;
        let destination = native_mods::catalog(&folder.expect("required by clap"), &destination)?;
        println!("{}", destination.display());
        return Ok(0);
    }
    if sync_internals {
        native_permission(trust_native_code)?;
        let entries = native_mods::sync_internals(&folder.expect("required by clap"))?;
        println!(
            "{}",
            serde_json::to_string(&entries).map_err(|error| error.to_string())?
        );
        return Ok(0);
    }
    let progress = native_progress(std::io::stderr().is_terminal());
    let steps = |step| {
        if let Some(label) = progress(step) {
            eprintln!("{label}");
        }
    };
    if let Some(output) = server_output {
        return export_server(
            folder,
            output,
            version,
            profile,
            allow_jar_downloads,
            trust_native_code,
            steps,
        );
    }
    build_profile(folder, instance, version, trust_native_code, profile, steps)
}

/// Export mode: a standalone server engine in `output`, from a mod `folder`.
fn export_server(
    folder: Option<PathBuf>,
    output: PathBuf,
    version: Option<String>,
    profile: Option<native_mods::Profile>,
    allow_jar_downloads: bool,
    trust_native_code: bool,
    steps: impl Fn(native_mods::Step),
) -> Result<i32, Error> {
    native_permission(trust_native_code)?;
    // Export mode: a standalone server engine in `output`. No instance is read
    // or changed, and no launcher config is written; `folder` is required (clap).
    let folder = folder.unwrap();
    let base = match &version {
        Some(v) => backend::resolve_version(Some(v))?,
        // The instance's version is never looked at here: the newest installed
        // release or Nightly, never a native build.
        None => native_mods::base_for(None, &backend::installed(), false)
            .ok_or_else(backend::no_version_installed)?,
    };
    let built =
        native_mods::build_server(&base, &folder, &output, profile, allow_jar_downloads, steps)?;
    let built = std::path::absolute(&built).unwrap_or(built);
    let path = built.display().to_string();
    println!(
        "{}",
        trf("Built the server engine: {path}", &[("path", &path)])
    );
    println!(
        "{}",
        tr(
            "Point the server at it: set engine.path to this file in the server's \
            server.toml (docs/hosting.md); the launcher writes nothing itself."
        )
    );
    Ok(0)
}

/// A profile's own build: its mods (or `folder`) against its game build, which it then plays.
fn build_profile(
    folder: Option<PathBuf>,
    instance: Option<String>,
    version: Option<String>,
    trust_native_code: bool,
    profile: Option<native_mods::Profile>,
    steps: impl Fn(native_mods::Step),
) -> Result<i32, Error> {
    let target = match &instance {
        Some(name) => instances::find(name)?,
        None => instances::selection()?,
    };
    let owner = if target.kind != instances::Kind::Pack
        && target.home.try_exists().map_err(|e| e.to_string())?
    {
        Some(sunrise_launcher::managed_play::Profile::open(
            &target.home,
            None,
        )?)
    } else {
        None
    };
    let active = owner
        .as_ref()
        .map(|owner| owner.active())
        .transpose()?
        .flatten();
    let base = match active {
        Some(active) => active.base().to_owned(),
        None => {
            let plays = target.resolve_version(None)?;
            let tag = native_mods::about(&plays).map_or(plays, |build| build.base);
            native_mods::base_for(Some(&tag), &backend::installed(), false)
                .ok_or_else(backend::no_version_installed)?
        }
    };
    check_version(version.as_deref(), &base)?;
    // A folder of source is code nobody reviewed. A profile's own mods build when the
    // registry verified them or the player confirmed them; the rest, such as files
    // copied in by hand, only when this run trusts them.
    if folder.is_some() {
        native_permission(trust_native_code)?;
    }
    if trust_native_code {
        trust_unreviewed_builds();
    }
    let target = match profile {
        Some(profile) => instances::edit(&target.id, |meta| meta.native_profile = Some(profile))?,
        None => target,
    };
    let explicit = folder.is_some() || profile.is_some();
    let folder = match folder {
        Some(folder) => folder,
        None => native_mods::installed_mods(&target.home)?,
    };
    let profile = explicit.then_some(target.native_profile);
    let built = match native_mods::build(
        &base,
        &folder,
        &target,
        profile,
        false,
        &native_mods::Cancellation::default(),
        steps,
        || backend::releases().ok(),
    ) {
        // A content mod needs no build: the instance stays as it is.
        Err(nothing @ native_mods::Failed::Nothing) => {
            println!("{nothing}");
            return Ok(0);
        }
        result => result,
    }
    .map_err(|error| build_failure(error, &base))?;
    println!("{built}");
    Ok(0)
}

/// The message for a failed build: what the worlds at stake keep, or how to pin a mod to `base`.
fn build_failure(error: native_mods::Failed, base: &str) -> String {
    if let native_mods::Failed::Worlds(stake) = &error {
        return trf(
            "{error}\n{stay}: nothing changed.",
            &[
                ("error", &stake.text),
                ("stay", &native_mods::fit::stay(&stake.stay)),
            ],
        );
    }
    let error = String::from(error);
    if error.starts_with(native_mods::MISMATCH) {
        trf("{error} Run `cargo sunrise-mod update --sunrise {commit}` in each mod's folder to pin it to this game, then build again.", &[("error", &error), ("commit", base)])
    } else {
        error
    }
}

/// `--version` for a profile asserts its current base, and never switches it.
fn check_version(version: Option<&str>, base: &str) -> Result<(), Error> {
    if let Some(requested) = version.filter(|tag| *tag != base) {
        return Err(trf(
            "This profile uses {build}, not {requested}. Use Switch to change its game build.",
            &[("build", base), ("requested", requested)],
        )
        .into());
    }
    Ok(())
}
