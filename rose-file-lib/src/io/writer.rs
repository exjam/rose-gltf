use std::cmp;
use std::io;
use std::io::{BufWriter, Seek, SeekFrom, Write};

use byteorder::{LittleEndian, WriteBytesExt};

use crate::error::RoseLibError;
use crate::utils::{Color3, Color4, Quaternion, Vector2, Vector3, Vector4};

/// Custom writers that supports some additional configurable options such
/// as writing strings as wide-strings.
//
// TODO: Add wide-string writing functionality
// TODO: Add tests (sample file: ai_s.stb)
pub struct RoseWriter<W: Write> {
    pub writer: BufWriter<W>,
}

impl<W: Write> RoseWriter<W> {
    pub fn new(inner: W) -> RoseWriter<W> {
        RoseWriter {
            writer: BufWriter::new(inner),
        }
    }
}

impl<W: Write> Write for RoseWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write + Seek> Seek for RoseWriter<W> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.writer.seek(pos)
    }
}

/// Extends `BufWriter` with methods for writing ROSE data types
///
///# Example
/// ```rust,no_run
/// use std::fs::File;
/// use std::io::BufWriter;
/// use rose_file_lib::io::WriteRoseExt;
///
/// let f = File::open("my_file.ext").unwrap();
/// let mut writer = BufWriter::new(f);
/// writer.write_i8(5i8).unwrap();
/// writer.write_f64(3.14f64).unwrap();
///
/// writer.write_cstring("null terminate me").unwrap();
/// ```
///
/// NOTE: Strings are encoded as UTF-8. The original ROSE files were encoded as EUC-KR,
/// as such if reading from an original file the data written back will be written using
/// a different encoding.
///
// Note: Clippy recommends passing by value for copy-able small args but
// we ignore that optimization in favor of API consistency
#[allow(clippy::trivially_copy_pass_by_ref)]
pub trait WriteRoseExt: Write + Seek {
    fn write_u8(&mut self, n: u8) -> Result<(), RoseLibError>;
    fn write_u16(&mut self, n: u16) -> Result<(), RoseLibError>;
    fn write_u32(&mut self, n: u32) -> Result<(), RoseLibError>;
    fn write_u64(&mut self, n: u64) -> Result<(), RoseLibError>;

    fn write_i8(&mut self, n: i8) -> Result<(), RoseLibError>;
    fn write_i16(&mut self, n: i16) -> Result<(), RoseLibError>;
    fn write_i32(&mut self, n: i32) -> Result<(), RoseLibError>;
    fn write_i64(&mut self, n: i64) -> Result<(), RoseLibError>;

    fn write_bool(&mut self, b: bool) -> Result<(), RoseLibError>;
    fn write_bool16(&mut self, b: bool) -> Result<(), RoseLibError>;

    fn write_f32(&mut self, n: f32) -> Result<(), RoseLibError>;
    fn write_f64(&mut self, n: f64) -> Result<(), RoseLibError>;

    // Write a fix-sized string
    fn write_string(&mut self, string: &str, len: i32) -> Result<(), RoseLibError>;

    // Write string as null terminated string
    fn write_cstring(&mut self, string: &str) -> Result<(), RoseLibError>;

    // Write a string with length prefix as u8
    fn write_string_u8(&mut self, string: &str) -> Result<(), RoseLibError>;

    // Write a string with length prefix as u16
    fn write_string_u16(&mut self, string: &str) -> Result<(), RoseLibError>;

    // Write a string with length prefix as u32
    fn write_string_u32(&mut self, string: &str) -> Result<(), RoseLibError>;

    /// Write a string with a variable-byte prefixed length to the reader
    ///
    /// If the string is less than 128 characters the the first byte holds the
    /// length. If the string is greater than or equal to128 characters then
    /// first two bytes hold the length.
    fn write_string_varbyte(&mut self, string: &str) -> Result<(), RoseLibError>;

    fn write_color3(&mut self, color: &Color3) -> Result<(), RoseLibError>;
    fn write_color4(&mut self, color: &Color4) -> Result<(), RoseLibError>;

    fn write_vector2_f32(&mut self, v: &Vector2<f32>) -> Result<(), RoseLibError>;
    fn write_vector2_i32(&mut self, v: &Vector2<i32>) -> Result<(), RoseLibError>;
    fn write_vector2_u32(&mut self, v: &Vector2<u32>) -> Result<(), RoseLibError>;

    fn write_vector3_f32(&mut self, v: &Vector3<f32>) -> Result<(), RoseLibError>;
    fn write_vector3_i16(&mut self, v: &Vector3<i16>) -> Result<(), RoseLibError>;
    fn write_vector4_f32(&mut self, v: &Vector4<f32>) -> Result<(), RoseLibError>;
    fn write_vector4_i16(&mut self, v: &Vector4<i16>) -> Result<(), RoseLibError>;

    fn write_quaternion(&mut self, q: &Quaternion) -> Result<(), RoseLibError>;
    fn write_quaternion_wxyz(&mut self, q: &Quaternion) -> Result<(), RoseLibError>;

    fn position(&mut self) -> Result<u64, RoseLibError>;
}

impl<W> WriteRoseExt for W
where
    W: Write + Seek + WriteBytesExt,
{
    fn write_u8(&mut self, n: u8) -> Result<(), RoseLibError> {
        WriteBytesExt::write_u8(self, n)?;
        Ok(())
    }

    fn write_u16(&mut self, n: u16) -> Result<(), RoseLibError> {
        WriteBytesExt::write_u16::<LittleEndian>(self, n)?;
        Ok(())
    }

    fn write_u32(&mut self, n: u32) -> Result<(), RoseLibError> {
        WriteBytesExt::write_u32::<LittleEndian>(self, n)?;
        Ok(())
    }

    fn write_u64(&mut self, n: u64) -> Result<(), RoseLibError> {
        WriteBytesExt::write_u64::<LittleEndian>(self, n)?;
        Ok(())
    }

    fn write_i8(&mut self, n: i8) -> Result<(), RoseLibError> {
        WriteBytesExt::write_i8(self, n)?;
        Ok(())
    }

    fn write_i16(&mut self, n: i16) -> Result<(), RoseLibError> {
        WriteBytesExt::write_i16::<LittleEndian>(self, n)?;
        Ok(())
    }

    fn write_i32(&mut self, n: i32) -> Result<(), RoseLibError> {
        WriteBytesExt::write_i32::<LittleEndian>(self, n)?;
        Ok(())
    }

    fn write_i64(&mut self, n: i64) -> Result<(), RoseLibError> {
        WriteBytesExt::write_i64::<LittleEndian>(self, n)?;
        Ok(())
    }

    fn write_bool(&mut self, b: bool) -> Result<(), RoseLibError> {
        let i = if b { 1u8 } else { 0u8 };
        WriteRoseExt::write_u8(self, i)?;
        Ok(())
    }

    fn write_bool16(&mut self, b: bool) -> Result<(), RoseLibError> {
        let i = if b { 1u16 } else { 0u16 };
        WriteRoseExt::write_u16(self, i)?;
        Ok(())
    }
    fn write_f32(&mut self, n: f32) -> Result<(), RoseLibError> {
        WriteBytesExt::write_f32::<LittleEndian>(self, n)?;
        Ok(())
    }

    fn write_f64(&mut self, n: f64) -> Result<(), RoseLibError> {
        WriteBytesExt::write_f64::<LittleEndian>(self, n)?;
        Ok(())
    }

    fn write_string(&mut self, string: &str, len: i32) -> Result<(), RoseLibError> {
        let string_len = string.len() as i32;

        let n_chars = cmp::min(string_len, len);
        for i in 0..n_chars {
            WriteRoseExt::write_u8(self, string.as_bytes()[i as usize])?;
        }

        if len > string_len {
            for _ in 0..(len - string_len) {
                WriteRoseExt::write_u8(self, 0x00)?;
            }
        }

        Ok(())
    }

    fn write_cstring(&mut self, string: &str) -> Result<(), RoseLibError> {
        self.write_all(string.as_bytes())?;
        WriteRoseExt::write_u8(self, 0x00)?;
        Ok(())
    }

    fn write_string_u8(&mut self, string: &str) -> Result<(), RoseLibError> {
        WriteRoseExt::write_u8(self, string.len() as u8)?;
        self.write_all(string.as_bytes())?;
        Ok(())
    }

    fn write_string_u16(&mut self, string: &str) -> Result<(), RoseLibError> {
        WriteRoseExt::write_u16(self, string.len() as u16)?;
        self.write_all(string.as_bytes())?;
        Ok(())
    }

    fn write_string_u32(&mut self, string: &str) -> Result<(), RoseLibError> {
        WriteRoseExt::write_u32(self, string.len() as u32)?;
        self.write_all(string.as_bytes())?;
        Ok(())
    }

    fn write_string_varbyte(&mut self, string: &str) -> Result<(), RoseLibError> {
        let len = string.len();
        if len < 128 {
            WriteRoseExt::write_u8(self, len as u8)?;
        } else {
            WriteRoseExt::write_u8(self, (len as u8) | 0b1000_0000)?;
            WriteRoseExt::write_u8(self, (len >> 7) as u8)?;
        }
        self.write_all(string.as_bytes())?;

        Ok(())
    }

    fn write_color3(&mut self, color: &Color3) -> Result<(), RoseLibError> {
        WriteRoseExt::write_f32(self, color.r)?;
        WriteRoseExt::write_f32(self, color.g)?;
        WriteRoseExt::write_f32(self, color.b)?;
        Ok(())
    }

    fn write_color4(&mut self, color: &Color4) -> Result<(), RoseLibError> {
        WriteRoseExt::write_f32(self, color.r)?;
        WriteRoseExt::write_f32(self, color.g)?;
        WriteRoseExt::write_f32(self, color.b)?;
        WriteRoseExt::write_f32(self, color.a)?;
        Ok(())
    }

    fn write_vector2_f32(&mut self, v: &Vector2<f32>) -> Result<(), RoseLibError> {
        WriteRoseExt::write_f32(self, v.x)?;
        WriteRoseExt::write_f32(self, v.y)?;
        Ok(())
    }

    fn write_vector2_i32(&mut self, v: &Vector2<i32>) -> Result<(), RoseLibError> {
        WriteRoseExt::write_i32(self, v.x)?;
        WriteRoseExt::write_i32(self, v.y)?;
        Ok(())
    }

    fn write_vector2_u32(&mut self, v: &Vector2<u32>) -> Result<(), RoseLibError> {
        WriteRoseExt::write_u32(self, v.x)?;
        WriteRoseExt::write_u32(self, v.y)?;
        Ok(())
    }

    fn write_vector3_f32(&mut self, v: &Vector3<f32>) -> Result<(), RoseLibError> {
        WriteRoseExt::write_f32(self, v.x)?;
        WriteRoseExt::write_f32(self, v.y)?;
        WriteRoseExt::write_f32(self, v.z)?;
        Ok(())
    }

    fn write_vector3_i16(&mut self, v: &Vector3<i16>) -> Result<(), RoseLibError> {
        WriteRoseExt::write_i16(self, v.x)?;
        WriteRoseExt::write_i16(self, v.y)?;
        WriteRoseExt::write_i16(self, v.z)?;
        Ok(())
    }

    fn write_vector4_f32(&mut self, v: &Vector4<f32>) -> Result<(), RoseLibError> {
        WriteRoseExt::write_f32(self, v.w)?;
        WriteRoseExt::write_f32(self, v.x)?;
        WriteRoseExt::write_f32(self, v.y)?;
        WriteRoseExt::write_f32(self, v.z)?;
        Ok(())
    }

    fn write_vector4_i16(&mut self, v: &Vector4<i16>) -> Result<(), RoseLibError> {
        WriteRoseExt::write_i16(self, v.w)?;
        WriteRoseExt::write_i16(self, v.x)?;
        WriteRoseExt::write_i16(self, v.y)?;
        WriteRoseExt::write_i16(self, v.z)?;
        Ok(())
    }

    fn write_quaternion(&mut self, q: &Quaternion) -> Result<(), RoseLibError> {
        WriteRoseExt::write_f32(self, q.x)?;
        WriteRoseExt::write_f32(self, q.y)?;
        WriteRoseExt::write_f32(self, q.z)?;
        WriteRoseExt::write_f32(self, q.w)?;
        Ok(())
    }

    fn write_quaternion_wxyz(&mut self, q: &Quaternion) -> Result<(), RoseLibError> {
        WriteRoseExt::write_f32(self, q.w)?;
        WriteRoseExt::write_f32(self, q.x)?;
        WriteRoseExt::write_f32(self, q.y)?;
        WriteRoseExt::write_f32(self, q.z)?;
        Ok(())
    }

    fn position(&mut self) -> Result<u64, RoseLibError> {
        Ok(self.stream_position()?)
    }
}
