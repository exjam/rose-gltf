//! ROSE Online String Table
use std::collections::HashMap;
use std::fmt;
use std::io::SeekFrom;
use std::str;
use std::str::FromStr;

use enum_map::{Enum, EnumMap};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};

use crate::error::RoseLibError;
use crate::io::{ReadRoseExt, RoseFile, WriteRoseExt};

/// String Table File
pub type STL = StringTable;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct StringTable {
    pub entry_type: StringTableType,
    pub entries: HashMap<String, StringTableEntry>,
}

/// String Table Type
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum StringTableType {
    #[default]
    Text,
    Description,
    Quest,
}

impl fmt::Display for StringTableType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StringTableType::Text => write!(f, "NRST01"),
            StringTableType::Description => write!(f, "ITST01"),
            StringTableType::Quest => write!(f, "QEST01"),
        }
    }
}

impl str::FromStr for StringTableType {
    type Err = RoseLibError;

    fn from_str(s: &str) -> Result<StringTableType, Self::Err> {
        match s {
            "NRST01" => Ok(StringTableType::Text),
            "ITST01" => Ok(StringTableType::Description),
            "QEST01" => Ok(StringTableType::Quest),
            _ => Err(RoseLibError::Generic(format!(
                "Unknown STL format identifier: {}",
                s
            ))),
        }
    }
}

#[derive(Debug, Copy, Clone, Default, Enum, Serialize, Deserialize, PartialEq, FromPrimitive)]
pub enum StringTableLanguage {
    Korean = 0,
    #[default]
    English = 1,
    Japanese = 2,
    ChineseTraditional = 3,
    ChineseSimplified = 4,
    Portuguese = 5,
    French = 6,
}

impl fmt::Display for StringTableLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StringTableLanguage::Korean => write!(f, "Korean"),
            StringTableLanguage::English => write!(f, "English"),
            StringTableLanguage::Japanese => write!(f, "Japanese"),
            StringTableLanguage::ChineseTraditional => write!(f, "Chinese (Traditional)"),
            StringTableLanguage::ChineseSimplified => write!(f, "Chinese (Simplified)"),
            StringTableLanguage::Portuguese => write!(f, "Portuguese"),
            StringTableLanguage::French => write!(f, "French"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum StringTableEntry {
    Text {
        text: EnumMap<StringTableLanguage, String>,
    },
    Description {
        text: EnumMap<StringTableLanguage, String>,
        description: EnumMap<StringTableLanguage, String>,
    },
    Quest {
        text: EnumMap<StringTableLanguage, String>,
        description: EnumMap<StringTableLanguage, String>,
        start: EnumMap<StringTableLanguage, String>,
        end: EnumMap<StringTableLanguage, String>,
    },
}

impl StringTableEntry {
    pub fn new(entry_type: StringTableType) -> Self {
        match entry_type {
            StringTableType::Text => Self::Text {
                text: Default::default(),
            },
            StringTableType::Description => Self::Description {
                text: Default::default(),
                description: Default::default(),
            },
            StringTableType::Quest => Self::Quest {
                text: Default::default(),
                description: Default::default(),
                start: Default::default(),
                end: Default::default(),
            },
        }
    }
}

impl RoseFile for StringTable {
    fn new() -> StringTable {
        Self::default()
    }

    fn read<R: ReadRoseExt>(&mut self, reader: &mut R) -> Result<(), RoseLibError> {
        self.entry_type = StringTableType::from_str(&reader.read_string_u8()?)?;

        let key_count = reader.read_u32()? as usize;
        let mut keys = Vec::with_capacity(key_count);
        for _ in 0..key_count {
            let string_key = reader.read_string_varbyte()?;
            let _integer_key = reader.read_u32()?;

            self.entries.insert(
                string_key.to_string(),
                StringTableEntry::new(self.entry_type),
            );
            keys.push(string_key.to_string());
        }

        let language_count = reader.read_u32()? as usize;
        for language_id in 0..language_count {
            let language_offset = reader.read_u32()?;
            let language_save_position = reader.position()?;

            let Some(language) = FromPrimitive::from_usize(language_id) else {
                continue;
            };

            reader.seek(SeekFrom::Start(language_offset as u64))?;

            for string_key in keys.iter() {
                let entry_offset = reader.read_u32()?;
                let entry_save_position = reader.position()?;
                reader.seek(SeekFrom::Start(entry_offset as u64))?;

                match self.entries.get_mut(string_key).unwrap() {
                    StringTableEntry::Text { text } => {
                        text[language] = reader.read_string_varbyte()?;
                    }
                    StringTableEntry::Description { text, description } => {
                        text[language] = reader.read_string_varbyte()?;
                        description[language] = reader.read_string_varbyte()?;
                    }
                    StringTableEntry::Quest {
                        text,
                        description,
                        start,
                        end,
                    } => {
                        text[language] = reader.read_string_varbyte()?;
                        description[language] = reader.read_string_varbyte()?;
                        start[language] = reader.read_string_varbyte()?;
                        end[language] = reader.read_string_varbyte()?;
                    }
                }

                reader.seek(SeekFrom::Start(entry_save_position))?;
            }

            reader.seek(SeekFrom::Start(language_save_position))?;
        }

        Ok(())
    }

    fn write<W: WriteRoseExt>(&mut self, writer: &mut W) -> Result<(), RoseLibError> {
        writer.write_string_varbyte(&self.entry_type.to_string())?;

        let mut keys = self.entries.keys().collect::<Vec<_>>();
        keys.sort_by(|lhs, rhs| human_sort::compare(lhs.as_str(), rhs.as_str()));

        writer.write_u32(keys.len() as u32)?;
        for (index, key) in keys.iter().enumerate() {
            writer.write_string_varbyte(key)?;
            writer.write_u32(index as u32)?;
        }

        writer.write_u32(StringTableLanguage::LENGTH as u32)?;
        let language_offsets = writer.position()?;
        for _ in 0..StringTableLanguage::LENGTH {
            writer.write_u32(0)?; // offset
        }

        let entry_offsets = writer.position()?;
        for language_index in 0..StringTableLanguage::LENGTH as u64 {
            let position = writer.position()?;
            writer.seek(SeekFrom::Start(language_offsets + 4 * language_index))?;
            writer.write_u32(position as u32)?;
            writer.seek(SeekFrom::Start(position))?;

            for _ in 0..keys.len() {
                writer.write_u32(0)?; // entry offset
            }
        }

        for (language_index, (language, _)) in EnumMap::<StringTableLanguage, ()>::default()
            .iter()
            .enumerate()
        {
            let language_entry_offsets = entry_offsets + (language_index * keys.len() * 4) as u64;
            for (entry_index, &key) in keys.iter().enumerate() {
                let position = writer.position()?;
                writer.seek(SeekFrom::Start(
                    language_entry_offsets + 4 * entry_index as u64,
                ))?;
                writer.write_u32(position as u32)?;
                writer.seek(SeekFrom::Start(position))?;

                match self.entries.get(key).unwrap() {
                    StringTableEntry::Text { text } => {
                        writer.write_string_varbyte(&text[language])?;
                    }
                    StringTableEntry::Description { text, description } => {
                        writer.write_string_varbyte(&text[language])?;
                        writer.write_string_varbyte(&description[language])?;
                    }
                    StringTableEntry::Quest {
                        text,
                        description,
                        start,
                        end,
                    } => {
                        writer.write_string_varbyte(&text[language])?;
                        writer.write_string_varbyte(&description[language])?;
                        writer.write_string_varbyte(&start[language])?;
                        writer.write_string_varbyte(&end[language])?;
                    }
                }
            }
        }

        Ok(())
    }
}
