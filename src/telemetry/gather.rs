//! Reports kept by a game played in an own profile. That game runs with the profile's folder as
//! its home (`instances::play::apply`), so it keeps its reports in the profile's own outbox. Only
//! the main game folder's outbox is sent, counted, shown and cleared, so they join it here.
use std::path::{Path, PathBuf};

use sunrise_launcher_telemetry::outbox;

/// Moves every report waiting in an own profile's outbox into the main game folder's.
pub fn gather() {
    let main = super::home();
    gather_into(&main, &profile_homes(&main));
}

/// Removes every report waiting in an own profile's outbox ("Delete my data").
pub(super) fn clear_profiles(main: &Path) -> std::io::Result<()> {
    profile_homes(main)
        .iter()
        .try_for_each(|home| outbox::clear(home))
}

fn profile_homes(main: &Path) -> Vec<PathBuf> {
    // A profile folder that can't be listed has nothing to gather that the next start won't find.
    crate::instances::profile_homes_strict(main).unwrap_or_default()
}

/// [`gather`] from `homes` into `main`. The main folder's answer decides what is kept, as it does
/// for the game's own reports, within the same daily caps. A report the main folder can't take
/// yet (it can't be written) stays where it is for the next time.
pub(super) fn gather_into(main: &Path, homes: &[PathBuf]) {
    for home in homes.iter().filter(|home| home.as_path() != main) {
        for (path, item) in outbox::pending(home) {
            // An item that can never be sent is not kept anywhere.
            if item.validate().is_err() || outbox::push(main, &item).is_ok() {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
}

#[cfg(test)]
mod tests;
