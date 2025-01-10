//! ROSE Online Data Table
use std::io::SeekFrom;

use serde::{Deserialize, Serialize};

use crate::error::RoseLibError;
use crate::io::{ReadRoseExt, RoseFile, WriteRoseExt};

/// Data File
pub type STB = DataTable;
pub type STBRow = DataTableRow;
pub type STBRowSlice<'a> = DataTableRowSlice<'a>;

// A row of columns
pub type DataTableRow = Vec<String>;
pub type DataTableRowSlice<'a> = &'a [String];

/// Data table column
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DataTableColumn {
    pub name: String,
    pub width: u16,
}

/// Data Table
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DataTable {
    pub identifier: String,

    pub header_row_height: u16,
    pub header_row_name: String,

    pub headers: Vec<DataTableColumn>,
    pub data: Vec<DataTableRow>,
    pub row_height: u16,
}

impl Default for DataTable {
    fn default() -> DataTable {
        DataTable {
            identifier: String::from("STB1"),

            header_row_height: 100,
            header_row_name: String::default(),

            headers: Vec::new(),
            data: Vec::new(),
            row_height: 40,
        }
    }
}

impl DataTable {
    pub fn rows(&self) -> usize {
        self.data.len()
    }

    pub fn cols(&self) -> usize {
        if self.rows() > 0 {
            self.data[0].len()
        } else {
            0
        }
    }

    pub fn column(&self, idx: usize) -> Option<&DataTableColumn> {
        if idx < self.headers.len() {
            return Some(&self.headers[idx]);
        }
        None
    }

    pub fn value(&self, row: usize, col: usize) -> Option<&str> {
        if row < self.rows() && col < self.cols() {
            return Some(&self.data[row][col]);
        }
        None
    }

    pub fn value_as_int(&self, row: usize, col: usize) -> Option<i32> {
        self.value(row, col).unwrap_or("").parse::<i32>().ok()
    }

    pub fn get(&self, row: usize, col: usize) -> &str {
        self.value(row, col).unwrap_or("")
    }

    pub fn get_int(&self, row: usize, col: usize) -> i32 {
        self.value_as_int(row, col).unwrap_or(0)
    }
}

impl RoseFile for DataTable {
    fn new() -> DataTable {
        Self::default()
    }

    fn read<R: ReadRoseExt>(&mut self, reader: &mut R) -> Result<(), RoseLibError> {
        self.identifier = reader.read_string(4)?;

        let offset = reader.read_u32()?;
        let row_count = (reader.read_u32()? as usize).saturating_sub(1);
        let col_count = (reader.read_u32()? as usize).saturating_sub(1);
        self.row_height = reader.read_u32()? as u16;

        let mut column_widths = Vec::with_capacity(col_count);
        let header_column_width;
        if self.identifier == "STB0" {
            self.header_row_height = 100;
            header_column_width = reader.read_u32()? as u16;
            column_widths.resize(col_count, header_column_width);
        } else {
            self.header_row_height = reader.read_u16()?;
            header_column_width = reader.read_u16()?;
            for _ in 0..col_count {
                column_widths.push(reader.read_u16()?);
            }
        }

        let header_column_name = reader.read_string_u16()?;
        self.headers.push(DataTableColumn {
            name: header_column_name,
            width: header_column_width,
        });
        #[allow(clippy::needless_range_loop)]
        for i in 0..col_count {
            self.headers.push(DataTableColumn {
                name: reader.read_string_u16()?,
                width: column_widths[i],
            });
        }

        self.header_row_name = reader.read_string_u16()?;
        for _ in 0..row_count {
            let row: Vec<String> = vec![reader.read_string_u16()?];
            self.data.push(row); // data[0] = row name
        }

        reader.seek(SeekFrom::Start(u64::from(offset)))?;

        for i in 0..row_count {
            for _ in 0..col_count {
                self.data[i].push(reader.read_string_u16()?);
            }
        }

        Ok(())
    }

    fn write<W: WriteRoseExt>(&mut self, writer: &mut W) -> Result<(), RoseLibError> {
        writer.write_string(&self.identifier, 4)?;

        // Write temporary offset
        writer.write_u32(0)?;

        writer.write_u32((self.data.len() + 1) as u32)?;
        writer.write_u32(self.headers.len() as u32)?;

        // Row height
        writer.write_u32(self.row_height as u32)?;

        // Root column width
        writer.write_u16(self.header_row_height)?;
        for header in &self.headers {
            writer.write_u16(header.width)?;
        }

        for header in &self.headers {
            writer.write_string_u16(&header.name)?;
        }

        // Unknown string
        writer.write_string_u16(&self.header_row_name)?;

        for row in &self.data {
            writer.write_string_u16(&row[0])?;
        }

        let offset = writer.stream_position()?;

        for row in &self.data {
            for cell in row.iter().skip(1) {
                writer.write_string_u16(cell)?;
            }
        }

        writer.seek(SeekFrom::Start(4))?;
        writer.write_u32(offset as u32)?;

        Ok(())
    }
}
