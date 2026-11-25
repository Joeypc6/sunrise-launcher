use super::*;
use sunrise_launcher_telemetry::payload::{Bug, Crash, Payload, Source};
use sunrise_launcher_telemetry::{Choices, Consent, Item};

fn allow(home: &Path, choices: Choices) {
    let mut consent = Consent::load(home);
    consent.answer(choices);
    consent.save(home).unwrap();
}

fn crash() -> Item {
    Item::new(Source::Game, "v1", Payload::Crash(Crash::default()))
}

fn bug() -> Item {
    let bug = Bug {
        text: "x".into(),
        ..Bug::default()
    };
    Item::new(Source::Game, "v1", Payload::Bug(bug))
}

#[test]
fn an_own_profiles_reports_join_the_main_queue_as_the_main_answer_allows() {
    let scratch = tempfile::tempdir().unwrap();
    let (main, own) = (scratch.path().join("game"), scratch.path().join("own"));
    allow(&own, Choices::ALL);
    assert!(outbox::push(&own, &crash()).unwrap());
    assert!(outbox::push(&own, &bug()).unwrap());
    // Since that game ran, the player turned bug reports off.
    allow(
        &main,
        Choices {
            crashes: true,
            ..Choices::NONE
        },
    );

    gather_into(&main, &[main.clone(), own.clone()]);

    let sent: Vec<_> = outbox::pending(&main).into_iter().map(|(_, i)| i).collect();
    assert_eq!(sent, [crash()]);
    assert!(outbox::pending(&own).is_empty());
}

#[test]
fn a_report_stays_in_its_profile_while_the_main_queue_cant_be_written() {
    let scratch = tempfile::tempdir().unwrap();
    let (main, own) = (scratch.path().join("game"), scratch.path().join("own"));
    allow(&own, Choices::ALL);
    allow(&main, Choices::ALL);
    assert!(outbox::push(&own, &crash()).unwrap());
    let queue = outbox::dir(&main);
    std::fs::write(&queue, b"not a folder").unwrap();

    gather_into(&main, std::slice::from_ref(&own));
    assert_eq!(outbox::pending(&own).len(), 1);

    std::fs::remove_file(&queue).unwrap();
    gather_into(&main, std::slice::from_ref(&own));
    assert!(outbox::pending(&own).is_empty());
    assert_eq!(outbox::pending(&main).len(), 1);
}

#[test]
fn delete_my_data_clears_every_own_profiles_queue() {
    let scratch = tempfile::tempdir().unwrap();
    let main = scratch.path();
    let own = main.join("instances").join("cabin");
    std::fs::create_dir_all(&own).unwrap();
    allow(&own, Choices::ALL);
    assert!(outbox::push(&own, &crash()).unwrap());
    clear_profiles(main).unwrap();
    assert!(outbox::pending(&own).is_empty());
}
