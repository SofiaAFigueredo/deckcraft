//! macOS: notice when DeckCraft runs from somewhere Finder can't make it the default app (#1305).
//!
//! Finder's Get Info › Change All records the app by its path. An app started from the mounted
//! disk image, or from Downloads while it is still quarantined, runs from a read-only `/Volumes/…`
//! path or from a random, temporary App Translocation path that is gone once the app quits. Open
//! With still works for a single file, but Change All fails with "You can't change the item …
//! to always open in the selected application (error code 2)". Copying the app to Applications
//! avoids it, so the app says so instead of leaving the user guessing.

use std::path::Path;

/// Where the running app lives, when that blocks making it the default for a file type.
#[derive(Debug, PartialEq, Eq)]
pub enum Misplaced {
    /// Started from the mounted disk image, or another volume that can be ejected.
    DiskImage,
    /// Gatekeeper's App Translocation: a quarantined app run from Downloads or the like.
    Translocated,
}

impl Misplaced {
    /// The status-bar text: what is wrong and what to do about it.
    pub fn message(&self) -> &'static str {
        match self {
            Self::DiskImage => {
                "DeckCraft is running from a disk image: drag it to Applications, eject the image and reopen it to make it the default app"
            }
            Self::Translocated => "DeckCraft is running from a temporary location: move it to Applications and reopen it to make it the default app",
        }
    }
}

/// Classify the path of the running executable (`…/DeckCraft.app/Contents/MacOS/DeckCraft`).
pub fn check(exe: &Path) -> Option<Misplaced> {
    let s = exe.to_str()?;
    if s.contains("/AppTranslocation/") {
        Some(Misplaced::Translocated)
    } else if s.starts_with("/Volumes/") {
        Some(Misplaced::DiskImage)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exe(p: &str) -> Option<Misplaced> {
        check(Path::new(p))
    }

    #[test]
    fn a_disk_image_is_misplaced() {
        assert_eq!(exe("/Volumes/DeckCraft/DeckCraft.app/Contents/MacOS/DeckCraft"), Some(Misplaced::DiskImage));
    }

    #[test]
    fn a_translocated_app_is_misplaced() {
        let p = "/private/var/folders/ab/xyz/T/AppTranslocation/6F1B/d/DeckCraft.app/Contents/MacOS/DeckCraft";
        assert_eq!(exe(p), Some(Misplaced::Translocated));
    }

    #[test]
    fn installed_apps_are_fine() {
        assert_eq!(exe("/Applications/DeckCraft.app/Contents/MacOS/DeckCraft"), None);
        assert_eq!(exe("/Users/a/Applications/DeckCraft.app/Contents/MacOS/DeckCraft"), None);
        assert_eq!(exe("/Users/a/src/deckcraft/target/release/deckcraft"), None);
    }

    #[test]
    fn every_case_has_advice() {
        for m in [Misplaced::DiskImage, Misplaced::Translocated] {
            assert!(m.message().contains("Applications"));
        }
    }
}
