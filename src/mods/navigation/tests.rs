use super::*;

fn open(id: &str, collection: bool, parent: Option<Parent>) -> Open {
    Open {
        id: id.into(),
        installed: None,
        collection,
        v2: true,
        request: 1,
        scope: OpenScope {
            profile: None,
            home: PathBuf::from("home"),
            target: None,
            game: "26.3".into(),
        },
        from: Page::Discover,
        v2_page: None,
        parent,
        removing: Vec::new(),
    }
}

/// #377: a dependency's page goes back to the mod it was listed on, and that mod's page then
/// back to the collection it was opened from.
#[test]
fn a_dependency_goes_back_to_the_mod_it_was_opened_from() {
    let factory = open(
        "factory-blocks",
        false,
        Some(Parent::Collection("builders".into())),
    );
    assert_eq!(
        factory.as_parent(),
        Some(Parent::Project {
            id: "factory-blocks".into(),
            // Before its page is read, the mod is named by its id.
            name: "factory-blocks".into(),
            parent: Some(Box::new(Parent::Collection("builders".into()))),
        })
    );
    // A collection's page lists no dependencies, so nothing returns to it this way.
    assert_eq!(open("builders", true, None).as_parent(), None);
    let installed_archive = Open {
        v2: false,
        ..open("local", false, None)
    };
    assert_eq!(installed_archive.as_parent(), None);
}
