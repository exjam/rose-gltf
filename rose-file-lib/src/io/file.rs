use std::fs::File;
use std::path::Path;

use crate::error::RoseLibError;
use crate::io::{ReadRoseExt, RoseReader, RoseWriter, WriteRoseExt};

pub trait RoseFile {
    /// Construct a new file
    ///
    /// # Example
    /// ```rust
    /// use rose_file_lib::files::ZMS;
    /// use rose_file_lib::io::RoseFile;
    ///
    /// let _ = ZMS::new();
    /// ```
    fn new() -> Self;

    /// Read data from a reader
    fn read<R: ReadRoseExt>(&mut self, reader: &mut R) -> Result<(), RoseLibError>;

    /// Write data to a writer
    fn write<W: WriteRoseExt>(&mut self, writer: &mut W) -> Result<(), RoseLibError>;

    /// Create new RoseFile from a `File`
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::fs::File;
    /// use rose_file_lib::files::ZMS;
    /// use rose_file_lib::io::RoseFile;
    ///
    /// let f = File::open("foo.zms").unwrap();
    /// let _ = ZMS::from_file(&f);
    /// ```
    fn from_file(file: &File) -> Result<Self, RoseLibError>
    where
        Self: Sized,
    {
        let mut rf = Self::new();
        rf.read_from_file(file)?;
        Ok(rf)
    }

    /// Create new RoseFile from a `Path`
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::path::PathBuf;
    /// use rose_file_lib::files::ZMS;
    /// use rose_file_lib::io::RoseFile;
    ///
    /// let p = PathBuf::from("/path/to/my.zms");
    /// let _ = ZMS::from_path(&p);
    /// ```
    fn from_path(path: &Path) -> Result<Self, RoseLibError>
    where
        Self: Sized,
    {
        let f = File::open(path).map_err(|source| RoseLibError::FileError {
            path: path.to_path_buf(),
            source,
        })?;

        Self::from_file(&f).map_err(|e| match e {
            RoseLibError::IOError(source) => RoseLibError::FileError {
                path: path.to_path_buf(),
                source,
            },
            _ => e,
        })
    }

    /// Read data from a `File`
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::fs::File;
    /// use rose_file_lib::files::ZMS;
    /// use rose_file_lib::io::RoseFile;
    ///
    /// let f = File::create("foo.zms").unwrap();
    /// let mut zms = ZMS::new();
    /// let _ = zms.read_from_file(&f);
    /// ```
    ///
    fn read_from_file(&mut self, file: &File) -> Result<(), RoseLibError> {
        let mut reader = RoseReader::new(file);
        self.read(&mut reader)?;
        Ok(())
    }

    /// Write data to a `File`
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::fs::File;
    /// use rose_file_lib::files::ZMS;
    /// use rose_file_lib::io::RoseFile;
    ///
    /// let f = File::create("foo.zms").unwrap();
    /// let mut zms = ZMS::new();
    /// let _ = zms.write_to_file(&f);
    /// ```
    fn write_to_file(&mut self, file: &File) -> Result<(), RoseLibError> {
        let mut writer = RoseWriter::new(file);
        self.write(&mut writer)?;
        Ok(())
    }

    /// Read data to the file from `Path`
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::path::PathBuf;
    /// use rose_file_lib::files::ZMS;
    /// use rose_file_lib::io::RoseFile;
    ///
    /// let p = PathBuf::from("/path/to/my.zms");
    /// let mut zms = ZMS::new();
    /// zms.read_from_path(&p);
    fn read_from_path(&mut self, path: &Path) -> Result<(), RoseLibError> {
        let f = File::open(path)?;
        let mut reader = RoseReader::new(f);
        self.read(&mut reader)?;
        Ok(())
    }

    /// Write data to a file at `Path`
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::path::PathBuf;
    /// use rose_file_lib::files::ZMS;
    /// use rose_file_lib::io::RoseFile;
    ///
    /// let p = PathBuf::from("/path/to/my.zms");
    /// let mut zms = ZMS::new();
    /// let _  = zms.write_to_path(&p);
    fn write_to_path(&mut self, path: &Path) -> Result<(), RoseLibError> {
        let f = File::create(path)?;
        self.write_to_file(&f)?;
        Ok(())
    }
}
