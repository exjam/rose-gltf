//! ROSE Scene
use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::error::RoseLibError;
use crate::io::{ReadRoseExt, RoseFile, WriteRoseExt};

/// Scene file
pub type CHR = CharacterModels;
pub type MON = CharacterModel;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum CharacterMotionType {
    Stop,
    Stop2,
    Move,
    Attack,
    Hit,
    Die,
    Run,
    SkillCast1,
    SkillAction1,
    SkillCast2,
    SkillAction2,
    Etc,
}

const ANI_MON_STOP1: u32 = 0x1;
const ANI_MON_STOP2: u32 = 0x2;
const ANI_MON_MOVE: u32 = 0x4;
const ANI_MON_ATTACK: u32 = 0x8;
const ANI_MON_HIT: u32 = 0x10;
const ANI_MON_DIE: u32 = 0x20;
const ANI_MON_RUN: u32 = 0x40;
const ANI_MON_SKILL1CASTING: u32 = 0x80;
const ANI_MON_SKILL1: u32 = 0x100;
const ANI_MON_SKILL2CASTING: u32 = 0x200;
const ANI_MON_SKILL2: u32 = 0x400;
const ANI_MON_ETC: u32 = 0x800;

impl CharacterMotionType {
    pub fn to_index(self) -> Option<u16> {
        match self {
            CharacterMotionType::Stop => Some(0),
            CharacterMotionType::Move => Some(1),
            CharacterMotionType::Attack => Some(2),
            CharacterMotionType::Hit => Some(3),
            CharacterMotionType::Die => Some(4),
            CharacterMotionType::Run => Some(5),
            CharacterMotionType::SkillCast1 => Some(6),
            CharacterMotionType::SkillAction1 => Some(7),
            CharacterMotionType::SkillCast2 => Some(8),
            CharacterMotionType::SkillAction2 => Some(9),
            CharacterMotionType::Etc => Some(10),
            CharacterMotionType::Stop2 => None,
        }
    }

    pub fn from_index(index: u16) -> Option<Self> {
        match index {
            0 => Some(Self::Stop),
            1 => Some(Self::Move),
            2 => Some(Self::Attack),
            3 => Some(Self::Hit),
            4 => Some(Self::Die),
            5 => Some(Self::Run),
            6 => Some(Self::SkillCast1),
            7 => Some(Self::SkillAction1),
            8 => Some(Self::SkillCast2),
            9 => Some(Self::SkillAction2),
            10 => Some(Self::Etc),
            _ => None,
        }
    }

    pub fn to_flags(self) -> Option<u32> {
        match self {
            CharacterMotionType::Stop => Some(ANI_MON_STOP1),
            CharacterMotionType::Stop2 => Some(ANI_MON_STOP2),
            CharacterMotionType::Move => Some(ANI_MON_MOVE),
            CharacterMotionType::Attack => Some(ANI_MON_ATTACK),
            CharacterMotionType::Hit => Some(ANI_MON_HIT),
            CharacterMotionType::Die => Some(ANI_MON_DIE),
            CharacterMotionType::Run => Some(ANI_MON_RUN),
            CharacterMotionType::SkillCast1 => Some(ANI_MON_SKILL1CASTING),
            CharacterMotionType::SkillAction1 => Some(ANI_MON_SKILL1),
            CharacterMotionType::SkillCast2 => Some(ANI_MON_SKILL2CASTING),
            CharacterMotionType::SkillAction2 => Some(ANI_MON_SKILL2),
            CharacterMotionType::Etc => Some(ANI_MON_ETC),
        }
    }

    pub fn from_flags(flags: u32) -> Option<Self> {
        if flags & ANI_MON_STOP1 != 0 {
            Some(Self::Stop)
        } else if flags & ANI_MON_STOP2 != 0 {
            Some(Self::Stop2)
        } else if flags & ANI_MON_MOVE != 0 {
            Some(Self::Move)
        } else if flags & ANI_MON_ATTACK != 0 {
            Some(Self::Attack)
        } else if flags & ANI_MON_HIT != 0 {
            Some(Self::Hit)
        } else if flags & ANI_MON_DIE != 0 {
            Some(Self::Die)
        } else if flags & ANI_MON_RUN != 0 {
            Some(Self::Run)
        } else if flags & ANI_MON_SKILL1CASTING != 0 {
            Some(Self::SkillCast1)
        } else if flags & ANI_MON_SKILL1 != 0 {
            Some(Self::SkillAction1)
        } else if flags & ANI_MON_SKILL2CASTING != 0 {
            Some(Self::SkillCast2)
        } else if flags & ANI_MON_SKILL2 != 0 {
            Some(Self::SkillAction2)
        } else if flags & ANI_MON_ETC != 0 {
            Some(Self::Etc)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CharacterModelMotion {
    pub animation: Option<String>,
    pub effect: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CharacterModel {
    pub name: String,
    pub skeleton_path: String,
    pub models: Vec<u16>,
    pub motions: HashMap<CharacterMotionType, String>,
    pub effects: Vec<(u16, String)>, // u16: dummy bone index
}

impl RoseFile for CharacterModel {
    fn new() -> CharacterModel {
        Self::default()
    }

    fn read<R: ReadRoseExt>(&mut self, reader: &mut R) -> Result<(), RoseLibError> {
        self.name = reader.read_string_u16()?;
        self.skeleton_path = reader.read_string_u16()?;

        let num_models = reader.read_u16()? as usize;
        self.models.reserve(num_models);
        for _ in 0..num_models {
            self.models.push(reader.read_u16()?);
        }

        let num_motions = reader.read_u16()? as usize;
        self.motions.reserve(num_motions);
        for _ in 0..num_motions {
            let motion_type_flags = reader.read_u32()?;
            let path = reader.read_string_u16()?;

            let Some(motion_type) = CharacterMotionType::from_flags(motion_type_flags) else {
                return Err(RoseLibError::Generic(format!(
                    "Invalid motion type flags {:X}",
                    motion_type_flags
                )));
            };
            self.motions.insert(motion_type, path);
        }

        let num_effects = reader.read_u16()? as usize;
        self.effects.reserve(num_effects);
        for _ in 0..num_effects {
            let dummy_bone_index = reader.read_u16()?;
            let _skill_index = reader.read_u16()?;
            let path = reader.read_string_u16()?;
            self.effects.push((dummy_bone_index, path));
        }

        Ok(())
    }

    fn write<W: WriteRoseExt>(&mut self, writer: &mut W) -> Result<(), RoseLibError> {
        writer.write_string_u16(&self.name)?;
        writer.write_string_u16(&self.skeleton_path)?;

        writer.write_u16(self.models.len() as u16)?;
        for model_id in self.models.iter().copied() {
            writer.write_u16(model_id)?;
        }

        writer.write_u16(self.motions.len() as u16)?;
        for (&motion_type, motion) in self.motions.iter() {
            writer.write_u32(motion_type.to_flags().ok_or(RoseLibError::Generic(format!(
                "Unsupported motion type {:?}",
                motion_type
            )))?)?;
            writer.write_string_u16(motion)?;
        }

        writer.write_u16(self.effects.len() as u16)?;
        for (dummy_bone_index, effect) in self.effects.iter() {
            writer.write_u16(*dummy_bone_index)?;
            writer.write_u16(0)?; // Unused: "Skill Index"
            writer.write_string_u16(effect)?;
        }

        Ok(())
    }
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct CharacterModels {
    pub models: Vec<Option<CharacterModel>>,
}

impl CharacterModels {
    pub fn get(&self, id: usize) -> Option<&CharacterModel> {
        self.models.get(id).and_then(|x| x.as_ref())
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut CharacterModel> {
        self.models.get_mut(id).and_then(|x| x.as_mut())
    }
}

impl RoseFile for CharacterModels {
    fn new() -> CharacterModels {
        Self::default()
    }

    fn read<R: ReadRoseExt>(&mut self, reader: &mut R) -> Result<(), RoseLibError> {
        let skeleton_file_count = reader.read_u16()? as usize;
        let mut skeleton_files = Vec::with_capacity(skeleton_file_count);
        for _ in 0..skeleton_file_count {
            skeleton_files.push(reader.read_cstring()?);
        }

        let motion_file_count = reader.read_u16()? as usize;
        let mut motion_files = Vec::with_capacity(motion_file_count);
        for _ in 0..motion_file_count {
            motion_files.push(reader.read_cstring()?);
        }

        let effect_file_count = reader.read_u16()? as usize;
        let mut effect_files = Vec::with_capacity(effect_file_count);
        for _ in 0..effect_file_count {
            effect_files.push(reader.read_cstring()?);
        }

        let character_count = reader.read_u16()? as usize;
        self.models.reserve(character_count);
        for _ in 0..character_count {
            if reader.read_u8()? == 0 {
                self.models.push(None);
                continue;
            }

            let skeleton_index = reader.read_u16()? as usize;
            let name = reader.read_cstring()?.to_string();

            let model_count = reader.read_u16()?;
            let mut models = Vec::new();
            for _ in 0..model_count {
                models.push(reader.read_u16()?);
            }

            let motion_count = reader.read_u16()?;
            let mut motions = HashMap::new();
            for _ in 0..motion_count {
                let motion_type_index = reader.read_i16()?;
                let motion_file_index = reader.read_u16()? as usize;

                if motion_type_index < 0 {
                    continue;
                }

                let Some(motion_type) = CharacterMotionType::from_index(motion_type_index as u16)
                else {
                    return Err(RoseLibError::Generic(format!(
                        "Invalid motion type index {}",
                        motion_type_index
                    )));
                };

                motions.insert(motion_type, motion_files[motion_file_index].clone());
            }

            let effect_count = reader.read_u16()?;
            let mut effects = Vec::new();
            for _ in 0..effect_count {
                let dummy_bone_index = reader.read_u16()?;
                let effect_file_index = reader.read_u16()? as usize;

                effects.push((dummy_bone_index, effect_files[effect_file_index].clone()));
            }

            self.models.push(Some(CharacterModel {
                name,
                skeleton_path: skeleton_files[skeleton_index].clone(),
                models,
                motions,
                effects,
            }));
        }

        Ok(())
    }

    fn write<W: WriteRoseExt>(&mut self, writer: &mut W) -> Result<(), RoseLibError> {
        let mut skeleton_files = self
            .models
            .iter()
            .filter_map(|x| x.as_ref())
            .map(|model| model.skeleton_path.clone())
            .collect::<HashSet<String>>()
            // Convert into a Vec
            .into_iter()
            .collect::<Vec<String>>();
        skeleton_files.sort();

        // Collect into a map of path -> index for later use
        let skeleton_files_map: HashMap<String, usize> = skeleton_files
            .iter()
            .enumerate()
            .map(|(x, y)| (y.clone(), x))
            .collect();

        // Collect all motion files
        let mut motion_files = self
            .models
            .iter()
            .filter_map(|x| x.as_ref())
            .flat_map(|model| model.motions.iter())
            .map(|(_, motion)| motion.clone())
            .collect::<HashSet<String>>()
            // Convert into a Vec
            .into_iter()
            .collect::<Vec<String>>();
        motion_files.sort();

        let motion_files_map: HashMap<String, usize> = motion_files
            .iter()
            .enumerate()
            .map(|(x, y)| (y.clone(), x))
            .collect();

        // Collect all motion files
        let mut effect_files = self
            .models
            .iter()
            .filter_map(|x| x.as_ref())
            .flat_map(|model| model.motions.iter())
            .map(|(_, effect)| effect.clone())
            .collect::<HashSet<String>>()
            // Convert into a Vec
            .into_iter()
            .collect::<Vec<String>>();
        effect_files.sort();

        let effect_files_map: HashMap<String, usize> = effect_files
            .iter()
            .enumerate()
            .map(|(x, y)| (y.clone(), x))
            .collect();

        writer.write_u16(skeleton_files.len() as u16)?;
        for path in skeleton_files.iter() {
            writer.write_cstring(path)?;
        }

        writer.write_u16(motion_files.len() as u16)?;
        for path in &motion_files {
            writer.write_cstring(path)?;
        }

        writer.write_u16(effect_files.len() as u16)?;
        for path in &effect_files {
            writer.write_cstring(path)?;
        }

        writer.write_u16(self.models.len() as u16)?;
        for model in self.models.iter() {
            let Some(model) = model else {
                writer.write_u8(0)?;
                continue;
            };
            writer.write_u8(1)?;

            writer.write_u16(*skeleton_files_map.get(&model.skeleton_path).unwrap() as u16)?;
            writer.write_cstring(&model.name)?;

            writer.write_u16(model.models.len() as u16)?;
            for model in model.models.iter().copied() {
                writer.write_u16(model)?;
            }

            writer.write_u16(model.motions.len() as u16)?;
            for (&motion_type, path) in model.motions.iter() {
                writer.write_u16(motion_type.to_index().ok_or(RoseLibError::Generic(
                    format!("Unsupported motion type {:?}", motion_type),
                ))?)?;
                writer.write_u16(*motion_files_map.get(path).unwrap() as u16)?;
            }

            writer.write_u16(model.effects.len() as u16)?;
            for (dummy_bone_index, path) in model.effects.iter() {
                writer.write_u16(*dummy_bone_index)?;
                writer.write_u16(*effect_files_map.get(path).unwrap() as u16)?;
            }
        }

        Ok(())
    }
}
