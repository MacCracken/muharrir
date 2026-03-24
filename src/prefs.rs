//! Preferences storage with JSON persistence and XDG path resolution.
//!
//! Extracted from shruti's preferences system. Provides [`PrefsStore`] for
//! loading/saving any `Serialize + Deserialize` type to a JSON file with
//! proper directory creation and Unix permission hardening.

use std::path::{Path, PathBuf};

/// Errors from preference operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PrefsError {
    /// I/O error reading or writing the preferences file.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON parse or serialization error.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Load/save any serializable preferences type from a JSON file.
///
/// Handles directory creation, pretty-printing, and Unix permission
/// hardening (0600). Consumers define their own preferences struct
/// with `#[derive(Serialize, Deserialize, Default)]` and use this
/// to persist it.
///
/// # Example (conceptual)
///
/// ```ignore
/// #[derive(Serialize, Deserialize, Default)]
/// struct MyPrefs { ui_scale: f32, theme: String }
///
/// let prefs: MyPrefs = PrefsStore::load_or_default(&path);
/// // ... modify prefs ...
/// PrefsStore::save(&prefs, &path)?;
/// ```
pub struct PrefsStore;

impl PrefsStore {
    /// Load preferences from a JSON file.
    pub fn load<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, PrefsError> {
        let data = std::fs::read_to_string(path)?;
        let prefs = serde_json::from_str(&data)?;
        tracing::debug!(path = %path.display(), "preferences loaded");
        Ok(prefs)
    }

    /// Save preferences to a JSON file.
    ///
    /// Creates parent directories if needed. On Unix, sets file
    /// permissions to 0600 (owner read/write only).
    pub fn save<T: serde::Serialize>(prefs: &T, path: &Path) -> Result<(), PrefsError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(prefs)?;
        std::fs::write(path, data.as_bytes())?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(path, perms)?;
        }

        tracing::debug!(path = %path.display(), "preferences saved");
        Ok(())
    }

    /// Load preferences, falling back to `Default` if the file doesn't exist
    /// or can't be parsed.
    pub fn load_or_default<T: serde::de::DeserializeOwned + Default>(path: &Path) -> T {
        match Self::load(path) {
            Ok(prefs) => prefs,
            Err(e) => {
                tracing::debug!(path = %path.display(), error = %e, "using default preferences");
                T::default()
            }
        }
    }
}

/// Resolve the platform-appropriate config directory for an application.
///
/// On Linux: `$XDG_CONFIG_HOME/<app_name>` or `~/.config/<app_name>`
/// Falls back to `./<app_name>` if no home directory is found.
#[must_use]
pub fn config_dir(app_name: &str) -> PathBuf {
    let base = if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config")
    } else {
        PathBuf::from(".")
    };
    base.join(app_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
    struct TestPrefs {
        ui_scale: f32,
        theme: String,
        max_undo: usize,
    }

    #[test]
    fn save_and_load() {
        let dir = std::env::temp_dir().join("muharrir_prefs_test_save_load");
        let path = dir.join("prefs.json");

        let prefs = TestPrefs {
            ui_scale: 1.5,
            theme: "dark".into(),
            max_undo: 100,
        };

        PrefsStore::save(&prefs, &path).unwrap();
        let loaded: TestPrefs = PrefsStore::load(&path).unwrap();
        assert_eq!(loaded, prefs);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_or_default_missing() {
        let path = Path::new("/tmp/muharrir_nonexistent_prefs_xyz.json");
        let prefs: TestPrefs = PrefsStore::load_or_default(path);
        assert_eq!(prefs, TestPrefs::default());
    }

    #[test]
    fn load_or_default_bad_json() {
        let dir = std::env::temp_dir().join("muharrir_prefs_test_bad_json");
        std::fs::create_dir_all(&dir).ok();
        let path = dir.join("bad.json");
        std::fs::write(&path, "not valid json{{{").unwrap();

        let prefs: TestPrefs = PrefsStore::load_or_default(&path);
        assert_eq!(prefs, TestPrefs::default());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn creates_parent_directories() {
        let dir = std::env::temp_dir().join("muharrir_prefs_test_nested");
        let path = dir.join("deep").join("nested").join("prefs.json");

        let prefs = TestPrefs::default();
        PrefsStore::save(&prefs, &path).unwrap();
        assert!(path.exists());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn unix_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::temp_dir().join("muharrir_prefs_test_perms");
        let path = dir.join("secure.json");

        PrefsStore::save(&TestPrefs::default(), &path).unwrap();

        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn config_dir_has_app_name() {
        let dir = config_dir("muharrir");
        assert!(dir.ends_with("muharrir"));
    }

    #[test]
    fn load_nonexistent_returns_error() {
        let result: Result<TestPrefs, _> = PrefsStore::load(Path::new("/no/such/file.json"));
        assert!(result.is_err());
    }

    #[test]
    fn prefs_error_display() {
        let err = PrefsError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "gone"));
        assert!(err.to_string().contains("io error"));
    }
}
