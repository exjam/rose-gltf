use std::path::{Path, PathBuf};

use crate::error::RoseLibError;

/// Extends `PathBuf` to read/write ROSE-style path strings
///
/// These methods enable us to serialize path strings such that we can
/// read/write the files across multiple platforms. All `\` separators are
/// converted to `/` separators instead. While there are
/// some limitations to this method (see below), this enables the best
/// cross-platform compatibility while still supporting the legacy files.
///
/// # Limitations
/// * Using absolute paths (e.g `C:\Foo\Bar`) on windows will cause unexpected
///   behaviour
/// * On unix-like systems, files/directories with `\` in their name will not
///   be considered a single path component (e.g. "my/home/iscool\\fun" =>
///   `["my", "home", "iscool", "fun"] NOT ["my", "home", "iscool\fun"])
///
pub trait PathRoseExt {
    /// Converts from a ROSE-syle path String to a PathBuf
    ///
    /// # Examples
    /// ```
    /// use std::path::PathBuf;
    /// use rose_file_lib::io::{PathRoseExt};
    ///
    /// let foo = PathBuf::from_rose_path("FOO/BAR\\BAZ");
    /// assert_eq!(foo.file_name().unwrap().to_str(), Some("BAZ"));
    /// ```
    fn from_rose_path(path_str: &str) -> PathBuf;

    /// Converts from a PathBuf to a ROSE-style path String
    ///
    /// # Examples
    /// ```
    /// use std::path::PathBuf;
    /// use rose_file_lib::io::{PathRoseExt};
    ///
    /// let path = PathBuf::from_rose_path("FOO/BAR\\BAZ");
    /// let str = path.to_rose_path();
    /// assert_eq!("FOO/BAR/BAZ", str);
    /// ```
    fn to_rose_path(&self) -> String;
}

impl PathRoseExt for PathBuf {
    fn from_rose_path(path_str: &str) -> PathBuf {
        let s = path_str.replace('\\', "/");
        PathBuf::from(s)
    }

    fn to_rose_path(&self) -> String {
        let mut s = String::new();
        let comp_count = self.components().count();

        for (idx, component) in self.iter().enumerate() {
            if let Some(c) = component.to_str() {
                s.push_str(c);
                if idx < comp_count - 1 {
                    s.push('/');
                }
            }
        }
        s
    }
}

/// Normalize a path string to a standard form
///
/// Lowercases the string, converts all backslash to forward slash (\ -> /) and
/// removes duplicate slashes
pub fn normalize_path_str(s: &str) -> Result<String, RoseLibError> {
    let mut s = s.to_lowercase().replace('\\', "/");

    while s.contains("//") {
        s = s.replace("//", "/");
    }

    Ok(s)
}

/// Normalize a path to a standard form
///
/// Lowercases the string, converts all backslash to forward slash (\ -> /) and
/// removes duplicate slashes
pub fn normalize_path(path: &Path) -> Result<String, RoseLibError> {
    let p = path
        .to_str()
        .ok_or(RoseLibError::Generic("Failed to normalize path".into()))?;

    normalize_path_str(p)
}
