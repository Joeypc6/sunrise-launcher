use super::*;
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        loop {
            let path = std::env::temp_dir().join(format!(
                "sunrise-location-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(checked_path(&fs::canonicalize(path).unwrap()).unwrap()),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => panic!("create fixture: {e}"),
            }
        }
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        // std removes links themselves rather than traversing their targets.
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, path: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            let meta = fs::symlink_metadata(&path).unwrap();
            reject_link(&path, &meta).unwrap(); // Never follow links while checking fixtures.
            if meta.is_dir() {
                visit(root, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(root).unwrap().into(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

struct Fixture {
    temp: Temp,
    source: Roots,
    bootstrap: PathBuf,
    pointer: Vec<u8>,
    original: BTreeMap<PathBuf, Vec<u8>>,
}
impl Fixture {
    fn new(build: &str) -> Self {
        let temp = Temp::new();
        let source = Roots {
            launcher: temp.0.join("source/launcher"),
            game: temp.0.join("source/game"),
            build: temp.0.join("source").join(build),
        };
        for (root, name, bytes) in [
            (&source.launcher, "accounts.json", &b"{\"accounts\":[]}"[..]),
            (
                &source.game,
                "saves/dummy/level.dat",
                &b"\0\xff\x80world\0"[..],
            ),
            (
                &source.build,
                "cache/compiled.bin",
                &b"\0\x01\xfe\xffbuild"[..],
            ),
        ] {
            let path = root.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, bytes).unwrap();
            fs::create_dir_all(root.join("empty")).unwrap();
        }
        let bootstrap = temp.0.join("bootstrap/location.json");
        fs::create_dir(bootstrap.parent().unwrap()).unwrap();
        let pointer = serde_json::to_vec(&Location {
            version: 1,
            roots: source.clone(),
        })
        .unwrap();
        fs::write(&bootstrap, &pointer).unwrap();
        let original = snapshot(&temp.0.join("source"));
        Self {
            temp,
            source,
            bootstrap,
            pointer,
            original,
        }
    }
    fn target(&self) -> PathBuf {
        self.temp.0.join("destination")
    }
    fn unchanged(&self) {
        assert_eq!(snapshot(&self.temp.0.join("source")), self.original);
        for root in [&self.source.launcher, &self.source.game, &self.source.build] {
            assert_eq!(fs::read_dir(root.join("empty")).unwrap().count(), 0);
        }
    }
    fn old_pointer(&self) {
        assert_eq!(fs::read(&self.bootstrap).unwrap(), self.pointer);
        assert_eq!(
            resolve_at(&self.bootstrap).unwrap(),
            Some(self.source.clone())
        );
        self.unchanged();
    }
    fn migrated(&self, roots: &Roots) {
        assert_eq!(resolve_at(&self.bootstrap).unwrap(), Some(roots.clone()));
        for (source, dest) in [
            (&self.source.launcher, &roots.launcher),
            (&self.source.game, &roots.game),
            (&self.source.build, &roots.build),
        ] {
            assert_eq!(snapshot(source), snapshot(dest));
            assert_eq!(fs::read_dir(dest.join("empty")).unwrap().count(), 0);
        }
        self.unchanged();
    }
    fn refuse(&self, target: &Path) -> String {
        let error = migrate(&self.source, target, &self.bootstrap, |_| {}).unwrap_err();
        self.old_pointer();
        error
    }
}

#[test]
fn full_move_preserves_source_and_commits_separate_build() {
    let f = Fixture::new("build");
    let target = f.target();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&f.source.launcher, fs::Permissions::from_mode(0o750)).unwrap();
        fs::set_permissions(
            f.source.launcher.join("accounts.json"),
            fs::Permissions::from_mode(0o640),
        )
        .unwrap();
    }
    let roots = migrate(&f.source, &target, &f.bootstrap, |_| {}).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for root in [&f.source.launcher, &roots.launcher] {
            assert_eq!(
                fs::metadata(root).unwrap().permissions().mode() & 0o777,
                0o750
            );
            assert_eq!(
                fs::metadata(root.join("accounts.json"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o640
            );
        }
    }
    assert_eq!(
        roots,
        Roots {
            launcher: target.join("launcher"),
            game: target.join("game"),
            build: target.join("build"),
        }
    );
    assert_ne!(fs::read(&f.bootstrap).unwrap(), f.pointer);
    f.migrated(&roots);
}

#[test]
fn full_move_preserves_aliased_and_nested_build_layouts() {
    for build in ["launcher", "launcher/native-build"] {
        let f = Fixture::new(build);
        let roots = migrate(&f.source, &f.target(), &f.bootstrap, |_| {}).unwrap();
        assert_eq!(roots.build, f.target().join(build));
        assert!(!f.target().join("build").exists());
        f.migrated(&roots);
    }
}

#[test]
fn nonempty_destination_preserves_pointer_and_contents() {
    let f = Fixture::new("build");
    fs::create_dir(f.target()).unwrap();
    fs::write(f.target().join("sentinel"), b"keep").unwrap();
    assert!(f.refuse(&f.target()).contains("empty folder"));
    assert_eq!(fs::read(f.target().join("sentinel")).unwrap(), b"keep");
}

#[test]
fn nested_same_and_ancestor_destinations_preserve_pointer() {
    let f = Fixture::new("build");
    for target in [
        f.source.game.join("nested"),
        f.source.launcher.clone(),
        f.temp.0.join("source"),
    ] {
        assert!(f.refuse(&target).contains("overlaps"));
    }
}

#[cfg(any(unix, windows))]
#[test]
fn source_entry_link_is_rejected_without_touching_its_target() {
    let f = Fixture::new("build");
    let outside = Temp::new();
    fs::write(outside.0.join("sentinel"), b"untouched").unwrap();
    let link = f.source.game.join("linked");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside.0, &link).unwrap();
    #[cfg(windows)]
    {
        let output = std::process::Command::new("cmd.exe")
            .args(["/d", "/c", "mklink", "/J"])
            .arg(link.to_string_lossy().replace('/', "\\"))
            .arg(outside.0.to_string_lossy().replace('/', "\\"))
            .output()
            .unwrap();
        assert!(output.status.success(), "junction setup: {output:?}");
    }
    struct Link(PathBuf);
    impl Drop for Link {
        fn drop(&mut self) {
            #[cfg(unix)]
            let _ = fs::remove_file(&self.0);
            #[cfg(windows)]
            let _ = fs::remove_dir(&self.0);
        }
    }
    let link = Link(link);
    let error = migrate(&f.source, &f.target(), &f.bootstrap, |_| {}).unwrap_err();
    drop(link);
    assert!(
        error.contains("Symbolic links and reparse points"),
        "{error}"
    );
    f.old_pointer();
    assert_eq!(fs::read(outside.0.join("sentinel")).unwrap(), b"untouched");
}

#[cfg(unix)]
struct PermissionsGuard(PathBuf, fs::Permissions);
#[cfg(unix)]
impl Drop for PermissionsGuard {
    fn drop(&mut self) {
        fs::set_permissions(&self.0, self.1.clone()).unwrap();
    }
}

#[cfg(any(unix, windows))]
#[test]
fn unreadable_source_keeps_old_pointer_and_source_bytes() {
    let f = Fixture::new("build");
    let path = f.source.game.join("saves/dummy/level.dat");
    #[cfg(windows)]
    let guard = {
        use std::os::windows::fs::OpenOptionsExt;
        OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&path)
            .unwrap()
    };
    #[cfg(unix)]
    let guard = {
        use std::os::unix::fs::PermissionsExt;
        unsafe extern "C" {
            fn geteuid() -> u32;
        }
        if unsafe { geteuid() } == 0 {
            eprintln!("SKIP unreadable_source_keeps_old_pointer_and_source_bytes: root bypasses chmod(000); run as an unprivileged user");
            return;
        }
        let guard = PermissionsGuard(path.clone(), fs::metadata(&path).unwrap().permissions());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o0)).unwrap();
        guard
    };
    assert!(
        File::open(&path).is_err(),
        "fixture must deny an actual read"
    );
    let mut committing = false;
    let error = migrate(&f.source, &f.target(), &f.bootstrap, |s| {
        committing |= s == "Committing data location"
    })
    .unwrap_err();
    drop(guard);
    assert!(!committing, "copy must fail before pointer commit");
    assert!(
        error.contains(&checked_path(&path).unwrap().display().to_string()),
        "{error}"
    );
    f.old_pointer();
}

#[test]
fn stale_worker_cannot_commit_after_successful_migration() {
    let f = Fixture::new("build");
    let roots = migrate(&f.source, &f.target(), &f.bootstrap, |_| {}).unwrap();
    let pointer = fs::read(&f.bootstrap).unwrap();
    let stale_target = f.temp.0.join("stale-worker");
    let error = migrate(&f.source, &stale_target, &f.bootstrap, |_| {}).unwrap_err();
    assert!(error.contains("Data location changed"), "{error}");
    assert_eq!(fs::read(&f.bootstrap).unwrap(), pointer);
    assert!(!stale_target.exists());
    f.migrated(&roots);
}

#[test]
fn shared_lock_blocks_migration_until_all_readers_release() {
    let f = Fixture::new("build");
    let first = acquire_lock_at(&f.bootstrap).unwrap();
    let second = acquire_lock_at(&f.bootstrap).unwrap();
    assert!(f.refuse(&f.target()).contains("Cannot lock"));
    drop(first);
    assert!(f.refuse(&f.target()).contains("Cannot lock"));
    assert!(!f.target().exists());
    drop(second);
    f.old_pointer();
    let roots = migrate(&f.source, &f.target(), &f.bootstrap, |_| {}).unwrap();
    f.migrated(&roots);
}

#[cfg(windows)]
#[test]
fn pointer_commit_failure_keeps_old_pointer_and_retries() {
    use std::os::windows::fs::OpenOptionsExt;
    let f = Fixture::new("build");
    // Allow reads/writes, but deny replacement of the existing pointer.
    let handle = OpenOptions::new()
        .read(true)
        .share_mode(1 | 2)
        .open(&f.bootstrap)
        .unwrap();
    let mut committing = false;
    let error = migrate(&f.source, &f.target(), &f.bootstrap, |s| {
        committing |= s == "Committing data location"
    })
    .unwrap_err();
    drop(handle);
    assert!(committing, "must fail during pointer commit: {error}");
    assert!(
        error.contains(&f.bootstrap.display().to_string()),
        "{error}"
    );
    f.old_pointer();
    let roots = migrate(&f.source, &f.temp.0.join("retry"), &f.bootstrap, |_| {}).unwrap();
    f.migrated(&roots);
}

#[cfg(windows)]
#[test]
fn canonical_windows_paths_are_validated_from_the_complete_root() {
    let path = std::fs::canonicalize(std::env::temp_dir()).unwrap();
    let plain = checked_path(&path).unwrap();
    assert_eq!(std::fs::canonicalize(&plain).unwrap(), path);
    assert_eq!(checked_path(&plain).unwrap(), plain);
}

#[test]
fn a_missing_or_unreadable_location_is_reported_before_the_window_opens() {
    let temp = Temp::new();
    let bootstrap = temp.0.join("location.json");
    let roots = Roots {
        launcher: temp.0.join("unplugged/launcher"),
        game: temp.0.join("unplugged/game"),
        build: temp.0.join("unplugged/build"),
    };
    let pointer = serde_json::to_vec(&Location {
        version: 1,
        roots: roots.clone(),
    })
    .unwrap();
    fs::write(&bootstrap, &pointer).unwrap();
    let error = startup_error_at(&bootstrap).expect("a missing folder must reach the window");
    assert!(error.contains("is missing"), "{error}");

    // A damaged pointer is the other way the launcher finds no data folder.
    fs::write(&bootstrap, b"not json").unwrap();
    let damaged = startup_error_at(&bootstrap).expect("a damaged pointer must reach the window");
    assert!(damaged.contains("Invalid"), "{damaged}");
}

#[test]
fn using_the_default_folder_keeps_the_pointer_beside_it() {
    let temp = Temp::new();
    let bootstrap = temp.0.join("location.json");
    let source = Roots {
        launcher: temp.0.join("source/launcher"),
        game: temp.0.join("source/game"),
        build: temp.0.join("source/build"),
    };
    for root in [&source.launcher, &source.game, &source.build] {
        fs::create_dir_all(root).unwrap();
    }
    let pointer = serde_json::to_vec(&Location {
        version: 1,
        roots: source.clone(),
    })
    .unwrap();
    fs::write(&bootstrap, &pointer).unwrap();

    use_default_at(&bootstrap).unwrap();

    assert!(!bootstrap.exists());
    assert_eq!(
        fs::read(temp.0.join("location.json.old")).unwrap(),
        pointer,
        "the old pointer stays recoverable"
    );
    for root in [&source.launcher, &source.game, &source.build] {
        assert!(root.is_dir(), "{} is not deleted", root.display());
    }
}

#[cfg(unix)]
#[test]
fn a_config_folder_behind_a_link_still_opens() {
    // A dotfiles-managed ~/.config is a link: players who never moved their data must start
    // without an error, and a pointer kept there must still resolve and lock.
    let temp = Temp::new();
    let real = temp.0.join("dotfiles/config");
    fs::create_dir_all(&real).unwrap();
    let config = temp.0.join("config");
    std::os::unix::fs::symlink(&real, &config).unwrap();
    let bootstrap = config.join("sunrise-launcher-location/location.json");

    assert_eq!(resolve_at(&bootstrap), Ok(None));
    assert_eq!(startup_error_at(&bootstrap), None);
    drop(acquire_lock_at(&bootstrap).unwrap());

    let roots = Roots {
        launcher: temp.0.join("data/launcher"),
        game: temp.0.join("data/game"),
        build: temp.0.join("data/build"),
    };
    for root in [&roots.launcher, &roots.game, &roots.build] {
        fs::create_dir_all(root).unwrap();
    }
    let pointer = serde_json::to_vec(&Location {
        version: 1,
        roots: roots.clone(),
    })
    .unwrap();
    fs::write(&bootstrap, pointer).unwrap();
    assert_eq!(resolve_at(&bootstrap), Ok(Some(roots)));

    // The lock behind the link is the one a worker takes through the real path.
    let reader = acquire_lock_at(&bootstrap).unwrap();
    assert!(lock_at(&real.join("sunrise-launcher-location/location.json"), true).is_err());
    drop(reader);
}

/// A link above the data folder (a ~/.local/share on another disk) is not refused: only the root
/// itself may not be one.
#[cfg(unix)]
#[test]
fn a_link_above_the_root_is_followed() {
    let base = std::env::temp_dir().join(format!("sunrise-dl-link-{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(base.join("disk/data")).unwrap();
    std::os::unix::fs::symlink(base.join("disk"), base.join("share")).unwrap();
    let checked = checked_path(&base.join("share/data")).unwrap();
    assert_eq!(checked, fs::canonicalize(base.join("disk/data")).unwrap());
    std::os::unix::fs::symlink(base.join("disk/data"), base.join("disk/linked")).unwrap();
    assert!(
        checked_path(&base.join("share/linked")).is_err(),
        "the root itself"
    );
    let _ = fs::remove_dir_all(&base);
}

/// No unit test reaches the player's real folders: a test's launcher data, build folders and
/// native build log are all under the temporary folder, or under the `SUNRISE_LAUNCHER_HOME` the
/// run names (launcher.yml points it at a scratch folder).
#[test]
fn unit_tests_never_resolve_the_real_profile() {
    // The data folder keeps the named spelling; the build folder is its real one.
    let homes = match std::env::var_os("SUNRISE_LAUNCHER_HOME").filter(|h| !h.is_empty()) {
        Some(home) => {
            let home = PathBuf::from(home);
            let real = crate::backend::plain(home.canonicalize().unwrap_or(home.clone()));
            vec![home, real]
        }
        None => vec![test_home()],
    };
    for path in [
        crate::backend::data_dir(),
        crate::backend::build_dir(),
        crate::native_mods::log_path(),
    ] {
        assert!(
            homes.iter().any(|home| path.starts_with(home)),
            "{} is outside {homes:?}",
            path.display()
        );
    }
}

#[test]
fn build_move_takes_only_build_entries_from_the_launcher_folder() {
    let f = Fixture::new("launcher");
    for name in [
        "rust/bin/rustc",
        "cargo/config.toml",
        "native/target/x.rlib",
    ] {
        let path = f.source.launcher.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, name).unwrap();
    }
    let before = snapshot(&f.temp.0.join("source"));
    let target = f.target();
    let moved = migrate_build(&f.source, &target, &f.bootstrap, |_| {}).unwrap();
    let roots = moved.roots.clone();
    assert_eq!(
        roots,
        Roots {
            launcher: f.source.launcher.clone(),
            game: f.source.game.clone(),
            build: target.clone(),
        }
    );
    assert_eq!(resolve_at(&f.bootstrap).unwrap(), Some(roots));
    for name in [
        "rust/bin/rustc",
        "cargo/config.toml",
        "native/target/x.rlib",
    ] {
        assert_eq!(fs::read(target.join(name)).unwrap(), name.as_bytes());
    }
    // Launcher data that is not a build file stays, and is not copied.
    assert!(!target.join("accounts.json").exists());
    assert!(!target.join("cache").exists());
    assert!(!target.join(".relocation-staging").exists());
    // The move copies; the old build files are the worker's to remove.
    assert_eq!(snapshot(&f.temp.0.join("source")), before);
    assert_eq!(
        moved.old,
        OldBuild::Entries(
            BUILD_ENTRIES
                .iter()
                .map(|name| f.source.launcher.join(name))
                .collect()
        )
    );
    assert!(moved.remove_old().is_empty());
    for name in BUILD_ENTRIES {
        assert!(!f.source.launcher.join(name).exists(), "{name}");
    }
    // Everything else in the launcher folder is kept.
    assert_eq!(
        fs::read(f.source.launcher.join("accounts.json")).unwrap(),
        b"{\"accounts\":[]}"
    );
    assert!(f.source.launcher.join("cache/compiled.bin").is_file());
    assert!(f.source.launcher.join("empty").is_dir());
}

#[test]
fn build_move_takes_a_separate_build_folder_whole() {
    let f = Fixture::new("build");
    let target = f.target();
    let moved = migrate_build(&f.source, &target, &f.bootstrap, |_| {}).unwrap();
    assert_eq!(moved.roots.build, target);
    assert_eq!(moved.roots.launcher, f.source.launcher);
    assert_eq!(snapshot(&f.source.build), snapshot(&target));
    assert_eq!(moved.old, OldBuild::Contents(f.source.build.clone()));
    f.unchanged();
    assert!(moved.remove_old().is_empty());
    // The folder itself stays, emptied; the launcher's and the game's data are untouched.
    assert_eq!(fs::read_dir(&f.source.build).unwrap().count(), 0);
    assert!(f.source.launcher.join("accounts.json").is_file());
    assert!(f.source.game.join("saves/dummy/level.dat").is_file());
}

#[test]
fn build_move_refuses_a_destination_in_the_data_or_not_empty() {
    let f = Fixture::new("launcher");
    for target in [
        f.source.launcher.join("builds"),
        f.source.game.join("builds"),
    ] {
        let error = migrate_build(&f.source, &target, &f.bootstrap, |_| {}).unwrap_err();
        assert!(error.contains("overlaps"), "{error}");
        f.old_pointer();
    }
    fs::create_dir(f.target()).unwrap();
    fs::write(f.target().join("sentinel"), b"keep").unwrap();
    let error = migrate_build(&f.source, &f.target(), &f.bootstrap, |_| {}).unwrap_err();
    assert!(error.contains("empty folder"), "{error}");
    assert_eq!(fs::read(f.target().join("sentinel")).unwrap(), b"keep");
    f.old_pointer();
}

/// Links `link` to the folder `to`: a symlink on Unix, a junction (no privilege needed) on
/// Windows.
fn link_folder(to: &Path, link: &Path) {
    #[cfg(unix)]
    std::os::unix::fs::symlink(to, link).unwrap();
    #[cfg(windows)]
    {
        let status = std::process::Command::new("cmd")
            .arg("/C")
            .arg("mklink")
            .arg("/J")
            .arg(link)
            .arg(to)
            .stdout(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "mklink /J {}", link.display());
    }
}

/// The launcher folder spelled through a link above it (a `~/.local/share` on another disk)
/// and the build folder spelled through its real path are one folder: the move and the
/// cleanup both take only the build entries, never the launcher's own data.
#[test]
fn build_cleanup_keeps_launcher_data_under_another_spelling() {
    let temp = Temp::new();
    let real = temp.0.join("disk");
    fs::create_dir_all(real.join("launcher")).unwrap();
    fs::create_dir_all(temp.0.join("game")).unwrap();
    link_folder(&real, &temp.0.join("share"));
    let source = Roots {
        launcher: temp.0.join("share/launcher"),
        game: temp.0.join("game"),
        build: real.join("launcher"),
    };
    let sentinels = ["accounts.json", "settings.json", "versions/26.3/client.bin"];
    for name in sentinels
        .iter()
        .chain(&["rust/bin/rustc", "native/target/x.rlib"])
    {
        let path = real.join("launcher").join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, name).unwrap();
    }
    // As spelled, the two look like separate folders: deciding from these would empty the
    // whole launcher folder.
    assert!(!build_is_launcher(&source));
    let bootstrap = temp.0.join("bootstrap/location.json");
    let target = temp.0.join("builds");

    let moved = migrate_build(&source, &target, &bootstrap, |_| {}).unwrap();
    assert!(matches!(moved.old, OldBuild::Entries(_)), "{:?}", moved.old);
    assert!(!target.join("accounts.json").exists());
    assert!(target.join("rust/bin/rustc").is_file());
    assert!(moved.remove_old().is_empty());
    for name in sentinels {
        assert_eq!(
            fs::read(real.join("launcher").join(name)).unwrap(),
            name.as_bytes(),
            "{name}"
        );
    }
    assert!(!real.join("launcher/rust").exists());
    assert!(!real.join("launcher/native").exists());
}

/// A portable pointer (`SUNRISE_LAUNCHER_LOCATION_FILE`) kept among the build files would be
/// removed with them after the move, and the next start would forget it: refused up front.
#[test]
fn build_move_refuses_a_pointer_among_the_build_files() {
    for (build, pointer) in [
        ("build", "build/location.json"),
        ("launcher", "launcher/rust/location.json"),
    ] {
        let f = Fixture::new(build);
        let bootstrap = f.temp.0.join("source").join(pointer);
        fs::create_dir_all(bootstrap.parent().unwrap()).unwrap();
        fs::write(&bootstrap, &f.pointer).unwrap();
        let error = migrate_build(&f.source, &f.target(), &bootstrap, |_| {}).unwrap_err();
        assert!(error.contains("location file"), "{error}");
        assert_eq!(fs::read(&bootstrap).unwrap(), f.pointer);
        assert_eq!(resolve_at(&bootstrap).unwrap(), Some(f.source.clone()));
        assert!(!f.target().exists());
    }
    // Beside the build entries, in the launcher folder itself, it is kept and the move goes on.
    let f = Fixture::new("launcher");
    let bootstrap = f.source.launcher.join("location.json");
    fs::write(&bootstrap, &f.pointer).unwrap();
    let moved = migrate_build(&f.source, &f.target(), &bootstrap, |_| {}).unwrap();
    assert!(moved.remove_old().is_empty());
    assert_eq!(resolve_at(&bootstrap).unwrap(), Some(moved.roots));
}
