use super::*;
use std::io::{BufRead, BufReader, Write};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sunrise-servers-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn folder_names_are_plain() {
    assert_eq!(slug("Ludo's Survival!"), "ludo-s-survival");
    assert_eq!(slug("  "), "server");
    assert_eq!(slug("Été 2026"), "t-2026");
    assert_eq!(service_suffix(Path::new("/x/My World")), "my-world");
}

#[test]
fn create_asks_init_and_config_set() {
    let spec = NewServer {
        name: "Friends".into(),
        world: World::New {
            seed: " 12345 ".into(),
        },
        players: vec!["Alex".into(), "Steve".into()],
        eula: true,
        check_accounts: false,
        ..NewServer::default()
    };
    assert_eq!(
        init_args(&spec, "world", 25566).join(" "),
        "init --yes --eula --world world --seed 12345 --port 25566 --whitelist --offline \
         --player Alex --player Steve --op Alex"
    );
    let open = NewServer {
        friends_only: false,
        world: World::Copy("/saves/Old".into()),
        ..NewServer::default()
    };
    assert_eq!(
        init_args(&open, "old", 25565).join(" "),
        "init --yes --world old --port 25565 --no-whitelist"
    );
    assert_eq!(
        settings_args(&spec, 8131).join(" "),
        "config set world.game_mode=survival world.difficulty=normal server.max_players=10 \
         server.motd=Friends panel.bind=127.0.0.1:8131"
    );
}

#[cfg(unix)]
#[test]
fn programs_are_laid_out_as_install_sh_does() {
    let root = scratch("runtime");
    let version = root.join("versions/v1");
    fs::create_dir_all(version.join(".local/sunrise-client/26.3")).unwrap();
    fs::write(version.join(PROGRAM), "server").unwrap();
    fs::write(version.join("sunrise-client"), "game").unwrap();
    fs::write(version.join(".installed"), "").unwrap();
    crate::game_app::ensure(&version).unwrap();
    fs::write(
        version.join(".local/sunrise-client/26.3/sunrise-resources.json"),
        "{}",
    )
    .unwrap();
    let rt = root.join("runtime");
    let resources = version.join(".local/sunrise-client/26.3");
    set_up_in(&rt, &version, "v1", "26.3", &resources).unwrap();
    assert_eq!(
        fs::read_to_string(rt.join("current").join(PROGRAM)).unwrap(),
        "server"
    );
    assert!(!rt.join("releases/v1/.installed").exists());
    // The game's macOS bundle is the launcher's, not the server's.
    assert!(!rt.join("releases/v1/Sunrise.app").exists());
    // The release finds the game files where sunrise-server looks: .local beside it.
    assert!(rt
        .join("current/.local/sunrise-client/26.3/sunrise-resources.json")
        .is_file());
    // Again with newer game files: only they change, swapped in whole.
    fs::write(
        version.join(".local/sunrise-client/26.3/sunrise-resources.json"),
        "{\"new\":1}",
    )
    .unwrap();
    set_up_in(&rt, &version, "v1", "26.3", &resources).unwrap();
    assert_eq!(
        fs::read_to_string(game_files(&rt, "26.3").join("sunrise-resources.json")).unwrap(),
        "{\"new\":1}"
    );
    assert!(!game_files(&rt, "26.3").with_file_name(".26.3.old").exists());
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn set_up_refuses_refresh_for_a_different_minecraft_version_than_active_runtime() {
    let root = scratch("runtime-version-mismatch");
    let old_version = root.join("versions/v1");
    fs::create_dir_all(old_version.join(".local/sunrise-client/26.3-pre-1")).unwrap();
    fs::write(old_version.join(PROGRAM), "server").unwrap();
    fs::write(
        old_version.join("research-lock.json"),
        r#"{"minecraft":{"version":"26.3-pre-1"}}"#,
    )
    .unwrap();
    let old_resources = old_version.join(".local/sunrise-client/26.3-pre-1");
    fs::write(old_resources.join("sunrise-resources.json"), "{}").unwrap();

    let rt = root.join("runtime");
    set_up_in(&rt, &old_version, "v1", "26.3-pre-1", &old_resources).unwrap();
    // Simulate missing game files for the active runtime while a newer build pins 26.3.
    fs::remove_dir_all(game_files(&rt, "26.3-pre-1")).unwrap();

    let new_version = root.join("versions/v2");
    let new_resources = new_version.join(".local/sunrise-client/26.3");
    fs::create_dir_all(&new_resources).unwrap();
    fs::write(new_version.join(PROGRAM), "server-new").unwrap();
    fs::write(
        new_version.join("research-lock.json"),
        r#"{"minecraft":{"version":"26.3"}}"#,
    )
    .unwrap();
    fs::write(new_resources.join("sunrise-resources.json"), "{}").unwrap();

    let error = set_up_in(&rt, &new_version, "v2", "26.3", &new_resources).unwrap_err();
    assert!(error.contains("26.3-pre-1"), "unexpected error: {error}");
    assert!(error.contains("26.3"), "unexpected error: {error}");
    assert!(
        !game_files(&rt, "26.3").exists(),
        "must not copy mismatched game files"
    );
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn native_server_setup_uses_base_resources_and_native_programs() {
    const ACTOR: &str = "SUNRISE_TEST_NATIVE_SERVER_SETUP";
    if std::env::var_os(ACTOR).is_none() {
        let home = tempfile::tempdir().unwrap();
        assert!(Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "servers::tests::native_server_setup_uses_base_resources_and_native_programs",
            ])
            .env(ACTOR, "1")
            .env("SUNRISE_LAUNCHER_HOME", home.path())
            .status()
            .unwrap()
            .success());
        return;
    }
    let base = "nightly-server-resources";
    let native = "native-server-resources";
    let base_dir = backend::version_dir(base);
    let native_dir = backend::version_dir(native);
    for dir in [&base_dir, &native_dir] {
        fs::create_dir_all(dir).unwrap();
        fs::write(dir.join(".installed"), "").unwrap();
    }
    fs::write(base_dir.join(PROGRAM), "base server").unwrap();
    fs::write(native_dir.join(PROGRAM), "native server").unwrap();
    let record = native_dir.join("native-mods.json");
    let write_record = |base: &str| {
        fs::write(
            &record,
            json!({"base": base, "sunrise": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "target": "client"}).to_string(),
        )
        .unwrap();
    };
    write_record(base);
    let lock = base_dir.join("research-lock.json");
    fs::write(
        &lock,
        json!({"minecraft": {"version": "26.3-pre-1"}}).to_string(),
    )
    .unwrap();
    // Native builds carry the base's lock into the runtime with their programs.
    fs::copy(&lock, native_dir.join("research-lock.json")).unwrap();
    // Only the base has resources; the native build need not duplicate them.
    assert!(can_host(native).is_err());
    // Not prepared yet: the one reason Create offers Prepare for (#530).
    assert!(matches!(set_up(native), Err(SetUpError::NotPrepared(_))));
    let resources = backend::resources(base);
    fs::create_dir_all(resources.join("nested")).unwrap();
    fs::write(resources.join("sunrise-resources.json"), "{}").unwrap();
    fs::write(resources.join("nested/data"), "base resources").unwrap();
    assert!(can_host(base).is_ok());
    assert!(can_host(native).is_ok());
    // A prepare already running is no reason to press Prepare, in any language.
    let (owner, root) = backend::installed_resources_root(native).unwrap();
    let preparing = backend::prepare_lock_at(&root, &owner).unwrap();
    let busy = set_up(native).unwrap_err();
    assert!(matches!(busy, SetUpError::Other(_)), "{busy:?}");
    assert_eq!(set_up_problem(busy).fix, None);
    drop(preparing);
    set_up(native).unwrap();
    assert!(game_files_ready());
    assert!(create_blocked(None).is_none());
    let default = game_files(&runtime_root(), backend::PINNED_GAME);
    fs::create_dir_all(&default).unwrap();
    fs::write(default.join("sunrise-resources.json"), "{}").unwrap();
    let current = game_files(&runtime_root(), "26.3-pre-1").join("sunrise-resources.json");
    fs::remove_file(&current).unwrap();
    assert!(
        !game_files_ready(),
        "another version's resources do not make the active release ready"
    );
    fs::write(&current, "{}").unwrap();
    assert_eq!(
        fs::read_to_string(program().unwrap()).unwrap(),
        "native server"
    );
    let copied = game_files(&runtime_root(), &backend::game_version(base));
    assert_eq!(
        fs::read_to_string(copied.join("nested/data")).unwrap(),
        "base resources"
    );
    assert!(!native_dir.join(".local/sunrise-client").exists());
    // Refresh an existing runtime from the same resolved base, too.
    fs::write(resources.join("nested/data"), "updated resources").unwrap();
    set_up(native).unwrap();
    assert_eq!(
        fs::read_to_string(copied.join("nested/data")).unwrap(),
        "updated resources"
    );
    assert!(!copied
        .with_file_name(format!(".{}.old", backend::game_version(base)))
        .exists());

    // Reject malformed base tags and names that resolve outside the installs directory.
    let outside = tempfile::tempdir().unwrap();
    fs::write(outside.path().join(".installed"), "").unwrap();
    for invalid in [
        "../invalid-base",
        outside.path().to_str().unwrap(),
        r"..\invalid-base",
    ] {
        write_record(invalid);
        assert!(can_host(native).unwrap_err().contains("native-mods.json"));
        assert!(set_up(native)
            .unwrap_err()
            .to_string()
            .contains("native-mods.json"));
    }
    let alias = backend::version_dir("base-symlink");
    point(&alias, outside.path()).unwrap();
    write_record("base-symlink");
    assert!(can_host(native).unwrap_err().contains("reinstall"));
    assert!(set_up(native)
        .unwrap_err()
        .to_string()
        .contains("reinstall"));
    write_record(base);
    assert_eq!(
        fs::read_to_string(program().unwrap()).unwrap(),
        "native server"
    );
    assert_eq!(
        fs::read_to_string(copied.join("nested/data")).unwrap(),
        "updated resources"
    );

    // A malformed game version stops setup before it modifies the runtime.
    let sibling = runtime_root().join("unrelated");
    fs::create_dir_all(&sibling).unwrap();
    let marker = sibling.join("keep");
    fs::write(&marker, "unrelated data").unwrap();
    for invalid in [
        "../../unrelated",
        sibling.to_str().unwrap(),
        r"..\..\unrelated",
    ] {
        fs::write(
            &lock,
            json!({"minecraft": {"version": invalid}}).to_string(),
        )
        .unwrap();
        assert!(can_host(native).unwrap_err().contains("research-lock.json"));
        assert!(set_up(native)
            .unwrap_err()
            .to_string()
            .contains("research-lock.json"));
        assert_eq!(fs::read_to_string(&marker).unwrap(), "unrelated data");
        assert_eq!(
            fs::read_to_string(copied.join("nested/data")).unwrap(),
            "updated resources"
        );
    }

    // Both an incomplete install and an entirely missing base call for reinstalling it.
    fs::remove_file(base_dir.join(".installed")).unwrap();
    assert!(set_up(native)
        .unwrap_err()
        .to_string()
        .contains("reinstall"));
    fs::remove_dir_all(&base_dir).unwrap();
    assert!(can_host(native).unwrap_err().contains("reinstall"));
    assert!(set_up(native)
        .unwrap_err()
        .to_string()
        .contains("reinstall"));
    assert!(game_files_ready());
    assert!(create_blocked(None).is_none());
}

#[cfg(unix)]
fn host_runtime_child(test: &str) -> bool {
    const CHILD: &str = "SUNRISE_TEST_HOST_RUNTIME";
    if std::env::var_os(CHILD).is_some() {
        return true;
    }
    let root = tempfile::tempdir().unwrap();
    assert!(Command::new(std::env::current_exe().unwrap())
        .args(["--exact", &format!("servers::tests::{test}")])
        .env(CHILD, "1")
        .env("SUNRISE_LAUNCHER_HOME", root.path().join("data"))
        .env("SUNRISE_HOME", root.path().join("game"))
        .status()
        .unwrap()
        .success());
    false
}

#[cfg(unix)]
#[test]
fn start_and_restart_publish_the_configured_resources_without_a_pending_field() {
    use std::os::unix::fs::PermissionsExt;
    use sunrise_launcher_smod::{
        locks::LockMode,
        runtime_leases::{open_directory, Registry},
    };
    if !host_runtime_child(
        "start_and_restart_publish_the_configured_resources_without_a_pending_field",
    ) {
        return;
    }
    let root = runtime_root();
    let release = root.join("releases/nightly");
    let data = backend::data_dir();
    let resources = data.join("configured game files");
    assert!(!game_files(&root, "26.3").exists());
    let server = data.join("servers/friends");
    for path in [&release, &resources, &server] {
        fs::create_dir_all(path).unwrap();
    }
    point(&root.join("current"), &release).unwrap();
    let program = release.join(PROGRAM);
    let reply = serde_json::json!({"format":1, "program":"engine", "args":["--resources", resources], "cwd":server, "resources":resources});
    fs::write(
        &program,
        format!("#!/bin/sh\n[ \"$3\" = engine-command ] || exit 1\ncat <<'EOF'\n{reply}\nEOF\n"),
    )
    .unwrap();
    fs::set_permissions(program, fs::Permissions::from_mode(0o755)).unwrap();
    let marker = release.join(".installed");
    command(&server, &["status"]).unwrap();
    assert!(!marker.exists(), "browsing must not modify a runtime");
    for verb in ["start", "restart"] {
        command(&server, &[verb]).unwrap();
        assert_eq!(fs::read(&marker).unwrap(), b"");
        fs::remove_file(&marker).unwrap();
    }
    let registry = Registry::existing(&data).unwrap();
    for path in [&release, &resources, &server] {
        for unit in path.ancestors().take_while(|p| *p != data) {
            registry
                .lock(&open_directory(unit).unwrap(), LockMode::Shared)
                .unwrap();
        }
    }
    let custom = data.parent().unwrap().join("custom");
    fs::create_dir(&custom).unwrap();
    fs::write(custom.join(PROGRAM), []).unwrap();
    point(&root.join("current"), &custom).unwrap();
    command(&server, &["start"]).unwrap();
    assert!(!custom.join(".installed").exists());
}

#[cfg(unix)]
#[test]
fn managed_start_prepares_pending_resources_before_publishing_and_starting() {
    use std::os::unix::fs::PermissionsExt;
    use sunrise_launcher_smod::{
        locks::LockMode,
        runtime_leases::{open_directory, Registry},
    };
    if !host_runtime_child(
        "managed_start_prepares_pending_resources_before_publishing_and_starting",
    ) {
        return;
    }
    let root = runtime_root();
    let release = root.join("releases/nightly");
    let data = backend::data_dir();
    let resources = game_files(&root, "26.3-server");
    let other_resources = game_files(&root, "26.3-other");
    let server = data.join("servers/friends");
    for path in [&release, &server] {
        fs::create_dir_all(path).unwrap();
    }
    point(&root.join("current"), &release).unwrap();
    let engine = release.join("engine");
    fs::write(&engine, []).unwrap();
    let reply = |path: &Path, pending| {
        json!({
            "format": 1, "program": engine, "args": ["--resources", path, "--dedicated"],
            "cwd": server, "resources": path, "resources_pending": pending,
        })
    };
    let program = release.join(PROGRAM);
    fs::write(
        &program,
        format!(
            r#"#!/bin/sh
set -eu
release=$(cd "$(dirname "$0")" && pwd -P)
resources="$release/../../game-files/sunrise-client/26.3-server"
other="$release/../../game-files/sunrise-client/26.3-other"
printf '%s\n' "$3" >> "$2/trace"
case "$3" in
    engine-command)
        if [ -d "$resources" ]; then
            cat <<'EOF'
{ready}
EOF
        elif [ -d "$other" ]; then
            cat <<'EOF'
{other_ready}
EOF
        else
            cat <<'EOF'
{pending}
EOF
        fi ;;
    init)
        [ "$4" = --yes ]
        [ "$#" -eq 4 ]
        [ ! -e "$release/.installed" ]
        [ ! -e "$resources" ]
        if [ -f "$2/init-fails" ]; then
            echo 'sunrise-server: preparation failed' >&2
            exit 1
        fi
        if [ -f "$2/changed-path" ]; then
            mkdir -p "$other"
        elif [ ! -f "$2/still-pending" ]; then
            mkdir -p "$resources"
        fi ;;
    start) [ -d "$resources" ] && [ -f "$release/.installed" ] ;;
    *) exit 1 ;;
esac
"#,
            ready = reply(&resources, false),
            other_ready = reply(&other_resources, false),
            pending = reply(&resources, true)
        ),
    )
    .unwrap();
    fs::set_permissions(program, fs::Permissions::from_mode(0o755)).unwrap();

    let start = || {
        run_managed(
            &server,
            &["start"],
            &|_| {},
            crate::jars::Permission::Refuse,
        )
    };
    let trace = server.join("trace");
    for (mode, error, verbs) in [
        ("init-fails", "Preparation failed", "engine-command\ninit\n"),
        (
            "still-pending",
            "The server's game files are still not prepared.",
            "engine-command\ninit\nengine-command\n",
        ),
        (
            "changed-path",
            "The server's game files are still not prepared.",
            "engine-command\ninit\nengine-command\n",
        ),
    ] {
        let flag = server.join(mode);
        fs::write(&flag, []).unwrap();
        let problem = start().unwrap_err();
        assert_eq!(problem.message, error);
        assert!(problem.fix.is_none());
        assert_eq!(fs::read_to_string(&trace).unwrap(), verbs);
        assert!(!resources.exists());
        assert!(!release.join(".installed").exists());
        if other_resources.exists() {
            fs::remove_dir_all(&other_resources).unwrap();
        }
        fs::remove_file(&trace).unwrap();
        fs::remove_file(flag).unwrap();
    }
    start().unwrap();
    assert_eq!(
        fs::read_to_string(trace).unwrap(),
        "engine-command\ninit\nengine-command\nstart\n"
    );
    let registry = Registry::existing(&data).unwrap();
    for path in [&release, &resources, &server] {
        registry
            .lock(&open_directory(path).unwrap(), LockMode::Shared)
            .unwrap();
    }
}

#[test]
fn problems_say_what_to_do() {
    let dir = scratch("problems");
    let port = Problem::explain_with(
        "",
        "[Server thread/WARN]: **** FAILED TO BIND TO PORT!\nThe exception was: Address already in use (os error 48)",
        &dir,
    );
    assert!(port.message.contains("already used"), "{}", port.message);
    assert!(matches!(port.fix, Some(Fix::UsePort(p)) if p > 25565));
    let files = Problem::explain_with(
        "sunrise-server: no prepared game files. Sunrise reads them from",
        "",
        &dir,
    );
    assert_eq!(files.fix, Some(Fix::Prepare));
    let reports = Problem::explain_with(
        "",
        "**** FAILED TO BIND TO PORT!\nThe exception was: registries.json not found next to the resources",
        &dir,
    );
    assert_eq!(reports.fix, Some(Fix::Prepare));
    let other = Problem::explain_with("sunrise-server: a server is already running in x", "", &dir);
    assert_eq!(other, Problem::plain("A server is already running in x"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn lists_and_status_read_as_vanilla_and_the_supervisor_write_them() {
    let dir = scratch("lists");
    fs::write(
        dir.join("whitelist.json"),
        r#"[{"uuid": "a", "name": "Alex"}, {"uuid": "b", "name": "Steve"}]"#,
    )
    .unwrap();
    assert_eq!(names(&dir, "whitelist.json"), ["Alex", "Steve"]);
    assert!(names(&dir, "ops.json").is_empty());
    fs::remove_dir_all(&dir).unwrap();

    let s = Status::parse(&json!({
        "phase": "running", "engine_uptime_seconds": 61, "crashes": 1, "last_crash": "signal: 9",
        "sample": { "rss_bytes": 800_000_000u64, "cpu_percent": 3.5, "game": {
            "tps": 20.0, "mspt": { "average": 4.2 }, "max_players": 10, "paused": false,
            "players": [{ "name": "Alex", "uuid": "u1", "ping": 12 }] } },
        "update": { "installed": "v1", "available": "v2", "busy": null, "last": "Not updated: x", "ok": false }
    }));
    assert_eq!(s.phase, "running");
    assert_eq!(s.players[0].name, "Alex");
    assert_eq!(s.players[0].ping, Some(12));
    assert_eq!(
        (s.tps, s.mspt, s.max_players),
        (Some(20.0), Some(4.2), Some(10))
    );
    assert_eq!(s.update_available.as_deref(), Some("v2"));
    assert_eq!(s.update_last, Some(("Not updated: x".into(), false)));
    assert!(!Status::stopped().running());
}

#[test]
fn hosting_uses_the_engine_startup_and_a_failed_poll_is_unknown() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(poll_hosting(dir.path(), None, 0).0.phase, "stopped");
    let lock_dir = dir.path().join(".sunrise");
    fs::create_dir(&lock_dir).unwrap();
    let lock = fs::File::create_new(lock_dir.join("lock")).unwrap();
    lock.try_lock().unwrap();
    assert_eq!(poll_hosting(dir.path(), None, 0).0.phase, "unknown");
    drop(lock);
    // A supervisor that died leaves its endpoint behind; with its lock free the server is
    // stopped, so Start stays available. Port 0 stands for the dead endpoint: nothing can
    // listen there, so the poll can't reach a server another test bound on a freed port.
    let address = "127.0.0.1:0";
    fs::write(
        lock_dir.join("api.json"),
        format!(r#"{{"address": "{address}", "token": "stale"}}"#),
    )
    .unwrap();
    assert_eq!(poll_hosting(dir.path(), None, 0).0.phase, "stopped");
    let lock = fs::File::open(lock_dir.join("lock")).unwrap();
    lock.try_lock().unwrap();
    assert_eq!(poll_hosting(dir.path(), None, 0).0.phase, "unknown");
    drop(lock);
    fs::remove_file(lock_dir.join("api.json")).unwrap();

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    fs::write(
        lock_dir.join("api.json"),
        json!({"address": listener.local_addr().unwrap().to_string(), "token":"test"}).to_string(),
    )
    .unwrap();
    // The file can already contain the next launch's port; it isn't evidence of what is open.
    fs::write(dir.path().join("server.properties"), "server-port=25567").unwrap();
    let status =
        |pid| json!({"phase":"running","pid":123,"engine_pid":pid,"engine_uptime_seconds":4});
    let log = |host| {
        json!({"next":3,"lines":[
            "[10:00:00] [Supervisor/INFO]: Starting game with the game files in resources",
            format!("[10:00:01] [Server thread/INFO]: Starting Minecraft server on {host}"),
            "[10:00:02] [Server thread/INFO]: Done (2s)!"
        ]})
    };
    let server = std::thread::spawn(move || {
        for (request, value) in [
            ("/api/v1/status", status(10)),
            ("/api/v1/logs?since=0", log("127.0.0.1:25566")),
            ("/api/v1/status", status(10)),
            ("/api/v1/status", status(20)),
            ("/api/v1/logs?since=3", json!({"next":3,"lines":[]})),
            ("/api/v1/logs?since=0", log("*:25567")),
            ("/api/v1/status", status(20)),
            ("/api/v1/status", json!({"error":"not available"})),
        ] {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(&mut stream);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert!(line.starts_with(&format!("GET {request} ")), "{line}");
            loop {
                line.clear();
                if reader.read_line(&mut line).unwrap() == 0 || line == "\r\n" {
                    break;
                }
            }
            let body = value.to_string();
            write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        }
    });
    // A live supervisor holds its lock, so a failed poll of it is unknown, not stopped.
    let lock = fs::File::open(lock_dir.join("lock")).unwrap();
    lock.try_lock().unwrap();
    let (first, _) = poll_hosting(dir.path(), None, 0);
    assert_eq!(first.listener, Some("127.0.0.1:25566".parse().unwrap()));
    let (second, _) = poll_hosting(dir.path(), Some(&first), 3);
    assert_eq!(second.listener, Some("0.0.0.0:25567".parse().unwrap()));
    let (unknown, _) = poll_hosting(dir.path(), Some(&second), 3);
    assert_eq!(unknown.phase, "unknown");
    assert!(unknown.listener.is_none());
    server.join().unwrap();
}

/// The control API: the token goes with every call, and errors come back as the server words them.
#[test]
fn the_api_sends_the_token() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap().to_string();
    let server = std::thread::spawn(move || {
        let mut seen = Vec::new();
        for reply in [
            r#"{"phase":"running","sample":{"game":{"players":[]}}}"#,
            r#"{"error":"the game didn't answer"}"#,
        ] {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut head = String::new();
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" || line.is_empty() {
                    break;
                }
                head.push_str(&line);
            }
            // Read the body too: closing with it unread would reset the connection.
            let length = head
                .lines()
                .find_map(|l| {
                    l.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .map(|n| n.trim().parse::<usize>().unwrap())
                })
                .unwrap_or(0);
            let mut body = vec![0; length];
            std::io::Read::read_exact(&mut reader, &mut body).unwrap();
            seen.push(head);
            let code = if reply.contains("error") {
                "409 Conflict"
            } else {
                "200 OK"
            };
            let mut stream = stream;
            write!(
                stream,
                "HTTP/1.1 {code}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{reply}",
                reply.len()
            )
            .unwrap();
        }
        seen
    });
    let dir = scratch("api");
    fs::create_dir_all(dir.join(".sunrise")).unwrap();
    fs::write(
        dir.join(".sunrise/api.json"),
        json!({ "address": address, "token": "t0k", "pid": 1 }).to_string(),
    )
    .unwrap();
    assert_eq!(status(&dir).phase, "running");
    let api = Api::open(&dir).unwrap();
    assert_eq!(
        api.command("list").unwrap_err(),
        "the game didn't answer".to_string()
    );
    let seen = server.join().unwrap();
    assert!(
        seen[0].starts_with("GET /api/v1/status HTTP/1.1"),
        "{}",
        seen[0]
    );
    assert!(seen
        .iter()
        .all(|h| h.to_ascii_lowercase().contains("authorization: bearer t0k")));
    assert!(seen[1].starts_with("POST /api/v1/command"));
    fs::remove_dir_all(&dir).unwrap();
    // Nothing answers: stopped.
    assert_eq!(status(Path::new("/nonexistent")).phase, "stopped");
}

#[test]
fn deleting_a_stopped_server_moves_its_world_and_backups_to_trash() {
    let root = scratch("delete-stopped");
    let dir = root.join("server");
    let trash = root.join("trash");
    fs::create_dir_all(dir.join("world/region")).unwrap();
    fs::create_dir_all(dir.join("backups")).unwrap();
    fs::write(dir.join("server.toml"), "[server]\n").unwrap();
    fs::write(dir.join("world/region/region.mca"), b"world").unwrap();
    fs::write(dir.join("backups/backup.zip"), b"backup").unwrap();
    trash_stopped_server(&dir, |path| crate::desktop_seams::to_folder(path, &trash)).unwrap();
    assert!(!dir.exists());
    assert_eq!(
        fs::read(trash.join("server/world/region/region.mca")).unwrap(),
        b"world"
    );
    assert_eq!(
        fs::read(trash.join("server/backups/backup.zip")).unwrap(),
        b"backup"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn deleting_refuses_a_live_supervisor_even_when_its_api_is_unavailable() {
    let root = scratch("delete-running");
    fs::create_dir_all(root.join(".sunrise")).unwrap();
    fs::write(root.join("server.toml"), "[server]\n").unwrap();
    let lock = fs::File::create(root.join(".sunrise/lock")).unwrap();
    lock.try_lock().unwrap();
    let result = trash_stopped_server(&root, |_| panic!("must not move a running server"));
    assert!(result.unwrap_err().contains("Stop the server"));
    assert!(root.join("server.toml").exists());
    drop(lock);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn deleting_refuses_an_unrecognized_folder_and_keeps_files_on_trash_failure() {
    let root = scratch("delete-invalid");
    assert!(trash_stopped_server(&root, |_| panic!("not a server")).is_err());
    fs::write(root.join("server.toml"), "[server]\n").unwrap();
    assert!(
        trash_stopped_server(&root, |_| Err("trash unavailable".into()))
            .unwrap_err()
            .contains("trash unavailable")
    );
    assert!(root.join("server.toml").exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn the_game_mode_comes_from_server_toml() {
    let dir = scratch("game-mode");
    assert_eq!(game_mode(&dir), None);
    fs::write(
        dir.join("server.toml"),
        "[server]\nport = 25565\n\n[world]\nname = \"world\"\ngame_mode = \"creative\"\n",
    )
    .unwrap();
    assert_eq!(game_mode(&dir).as_deref(), Some("creative"));
    fs::write(dir.join("server.toml"), "[server]\nport = 25565\n").unwrap();
    assert_eq!(game_mode(&dir), None);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn the_engine_command_reply_becomes_the_command_and_its_problems() {
    let reply = br#"{"format": 1, "program": "/opt/sunrise/sunrise-engine", "args": ["--dedicated", "--nogui"],
        "cwd": "/srv/mc", "env": {"SUNRISE_NO_AUDIO": "1", "GONE": null}, "resources": "/srv/mc/resources"}"#;
    let (engine, resources) = engine_from_reply(reply).unwrap().unwrap();
    assert_eq!(resources.path, Path::new("/srv/mc/resources"));
    assert!(!resources.pending);
    assert_eq!(engine.get_program(), "/opt/sunrise/sunrise-engine");
    assert_eq!(
        engine.get_args().collect::<Vec<_>>(),
        ["--dedicated", "--nogui"]
    );
    assert_eq!(engine.get_current_dir(), Some(Path::new("/srv/mc")));
    let env: Vec<_> = engine.get_envs().collect();
    assert!(env.contains(&(
        std::ffi::OsStr::new("SUNRISE_NO_AUDIO"),
        Some(std::ffi::OsStr::new("1"))
    )));
    assert!(env.contains(&(std::ffi::OsStr::new("GONE"), None)));

    let missing =
        br#"{"format": 1, "error": {"kind": "resources", "message": "no prepared game files"}}"#;
    let problem = engine_from_reply(missing).unwrap_err();
    assert_eq!(
        (problem.message.as_str(), problem.fix),
        ("no prepared game files", Some(Fix::Prepare))
    );
    let other =
        br#"{"format": 1, "error": {"kind": "configuration", "message": "server.toml: bad"}}"#;
    assert_eq!(engine_from_reply(other).unwrap_err().fix, None);

    // Builds from before engine-command: a usage line (exit 1, "unknown command"), nothing, or no object.
    for usage in [
        &b"error: unknown command engine-command\nusage: sunrise-server ..."[..],
        b"",
        b"[]",
    ] {
        assert!(engine_from_reply(usage).unwrap().is_none());
    }
    let odd = engine_from_reply(
        br#"{"format": 1, "program": "x", "args": ["--dedicated", 7], "cwd": "/"}"#,
    );
    assert!(odd.unwrap_err().message.contains("can't read"));
    let newer = engine_from_reply(br#"{"format": 2, "program": "x"}"#).unwrap_err();
    assert!(
        newer.message.contains("update the launcher"),
        "{}",
        newer.message
    );
}

/// A world folder whose level.dat (plain NBT) holds `data`; written after `pause` so its file
/// time is later than the folders before it.
fn saved_world(home: &Path, folder: &str, name: &str, last_played: i64) {
    let dir = home.join("saves").join(folder);
    fs::create_dir_all(&dir).unwrap();
    let data = sunrise_launcher_nbt::Tag::compound([(
        "Data",
        sunrise_launcher_nbt::Tag::compound([
            ("LevelName", sunrise_launcher_nbt::Tag::String(name.into())),
            ("LastPlayed", sunrise_launcher_nbt::Tag::Long(last_played)),
        ]),
    )]);
    let root = sunrise_launcher_nbt::NamedTag {
        name: "".into(),
        tag: data,
    };
    let bytes =
        sunrise_launcher_nbt::write_named(&root, sunrise_launcher_nbt::Limits::default()).unwrap();
    fs::write(dir.join("level.dat"), bytes).unwrap();
}

#[test]
fn the_copy_list_names_worlds_as_the_game_does_and_orders_by_last_played() {
    let home = scratch("player-worlds");
    // Written last, so the newest file time, but played the longest ago.
    saved_world(
        &home,
        "played-long-ago",
        "Renamed in game",
        1_000_000_000_000,
    );
    saved_world(&home, "played-today", "Today", 1_700_000_000_000);
    // A folder without level.dat is no world.
    fs::create_dir_all(home.join("saves/empty")).unwrap();
    std::thread::sleep(Duration::from_millis(20));
    saved_world(&home, "restored", "Restored", 1_500_000_000_000);
    let names: Vec<String> = player_worlds(&home).into_iter().map(|w| w.0).collect();
    assert_eq!(names, ["Today", "Restored", "Renamed in game"]);
    let first = player_worlds(&home).remove(0).1;
    assert_eq!(first, home.join("saves/played-today"));
    fs::remove_dir_all(home).unwrap();
}

#[test]
fn the_copy_list_leaves_out_folders_without_a_readable_level_dat() {
    let home = scratch("player-worlds-broken");
    saved_world(&home, "good", "Good", 1_000_000_000_000);
    std::thread::sleep(Duration::from_millis(20));
    // Both newer by file time than the good world, so either would be preselected if listed.
    let old_only = home.join("saves/old-only");
    fs::create_dir_all(&old_only).unwrap();
    fs::write(old_only.join("level.dat_old"), b"not nbt").unwrap();
    std::thread::sleep(Duration::from_millis(20));
    let garbage = home.join("saves/garbage");
    fs::create_dir_all(&garbage).unwrap();
    fs::write(garbage.join("level.dat"), b"not nbt").unwrap();
    let listed = player_worlds(&home);
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].1, home.join("saves/good"));
    fs::remove_dir_all(home).unwrap();
}

#[test]
fn create_offers_prepare_only_for_game_files_not_yet_prepared() {
    let not_prepared = SetUpError::NotPrepared("Sunrise v1 isn't prepared yet".into());
    assert_eq!(set_up_problem(not_prepared).fix, Some(Fix::Prepare));
    let busy = SetUpError::Other("Sunrise v1 is already being prepared.".into());
    assert_eq!(set_up_problem(busy).fix, None);
    // A copy error whose path happens to say "prepared" is still only a copy error.
    let copy = SetUpError::Other("/home/me/prepared worlds/server: Permission denied".into());
    assert_eq!(set_up_problem(copy).fix, None);
}
