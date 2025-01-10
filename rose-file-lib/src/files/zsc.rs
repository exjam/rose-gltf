//! ROSE Scene
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::io::SeekFrom;
use std::str::FromStr;

use bitflags::bitflags;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};

use crate::error::RoseLibError;
use crate::io::{ReadRoseExt, RoseFile, WriteRoseExt};
use crate::utils::{BoundingBox, BoundingCylinder, Color3, Quaternion, Vector2, Vector3};

pub type ZSC = ModelList;
pub type ZSCTXT = Model;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ModelList {
    pub models: Vec<Option<Model>>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Model {
    pub bounding_cylinder: BoundingCylinder,
    pub bounding_box: BoundingBox<f32>,
    pub parts: Vec<ModelPart>,
    pub dummy_points: Vec<ModelDummyPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialOrd)]
pub struct ModelMaterial {
    pub path: String,
    pub is_skin: bool,
    pub alpha_enabled: bool,
    pub two_sided: bool,
    pub alpha_test: Option<u8>,
    pub z_write_enabled: bool,
    pub z_test_enabled: bool,
    pub blend_mode: Option<MaterialBlendMode>,
    pub specular_enabled: bool,
    pub alpha: f32,
    pub glow: Option<MaterialGlow>,
}

impl PartialEq for ModelMaterial {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
            && self.is_skin == other.is_skin
            && self.alpha_enabled == other.alpha_enabled
            && self.two_sided == other.two_sided
            && self.alpha_test == other.alpha_test
            && self.z_write_enabled == other.z_write_enabled
            && self.z_test_enabled == other.z_test_enabled
            && self.blend_mode == other.blend_mode
            && self.specular_enabled == other.specular_enabled
            && ((self.alpha * 255.0) as i32) == ((other.alpha * 255.0) as i32)
            && self.glow == other.glow
    }
}

impl Eq for ModelMaterial {}

impl std::hash::Hash for ModelMaterial {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path.hash(state);
        self.is_skin.hash(state);
        self.alpha_enabled.hash(state);
        self.two_sided.hash(state);
        self.alpha_test.hash(state);
        self.z_write_enabled.hash(state);
        self.z_test_enabled.hash(state);
        self.blend_mode.hash(state);
        self.specular_enabled.hash(state);
        ((self.alpha * 255.0) as i32).hash(state);
        self.glow.hash(state);
    }
}

impl Default for ModelMaterial {
    fn default() -> Self {
        Self {
            path: String::default(),
            is_skin: false,
            alpha_enabled: false,
            two_sided: false,
            alpha_test: Some(128),
            z_write_enabled: true,
            z_test_enabled: true,
            blend_mode: None,
            specular_enabled: false,
            alpha: 1.0,
            glow: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ModelPart {
    pub mesh_path: String,
    pub material: Option<ModelMaterial>,
    pub position: Vector3<f32>,
    pub rotation: Quaternion,
    pub scale: Vector3<f32>,
    pub bone_index: Option<u16>,
    pub dummy_index: Option<u16>,
    pub parent: Option<u16>,
    pub collision_shape: Option<ModelCollisionShape>,
    pub collision_flags: ModelCollisionFlags,
    pub animation_path: Option<String>,
    pub range_set_id: Option<u16>, // Index of row in 3ddata/stb/rangeset.stb
    pub use_lightmap: bool,
}

impl Default for ModelPart {
    fn default() -> Self {
        Self {
            mesh_path: String::default(),
            material: Some(ModelMaterial::default()),
            position: Vector3::ZERO,
            rotation: Quaternion::IDENTITY,
            scale: Vector3::ONE,
            bone_index: None,
            dummy_index: None,
            parent: None,
            collision_shape: None,
            collision_flags: Default::default(),
            animation_path: None,
            range_set_id: None,
            use_lightmap: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ModelDummyPoint {
    pub attachment: Option<ModelDummyAttachment>,
    pub position: Vector3<f32>,
    pub rotation: Quaternion,
    pub scale: Vector3<f32>,
    pub parent: Option<u16>,
}

impl Default for ModelDummyPoint {
    fn default() -> Self {
        Self {
            attachment: None,
            position: Vector3::ZERO,
            rotation: Quaternion::IDENTITY,
            scale: Vector3::ONE,
            parent: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ModelDummyAttachment {
    Effect {
        path: String,
        only_visible_at_night: bool,
    },
    Light {
        name: String,
    },
}

#[derive(
    Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, FromPrimitive,
)]
pub enum MaterialBlendMode {
    Lighten = 1,
}

#[derive(
    Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, FromPrimitive,
)]
pub enum MaterialGlowType {
    Simple = 2,
    Light = 3,
    Texture = 4,
    TextureLight = 5,
    Alpha = 6,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialOrd)]
pub struct MaterialGlow {
    pub glow_type: MaterialGlowType,
    pub color: Color3,
}

impl PartialEq for MaterialGlow {
    fn eq(&self, other: &Self) -> bool {
        self.glow_type == other.glow_type
            && ((self.color.r * 255.0) as i32) == ((other.color.r * 255.0) as i32)
            && ((self.color.g * 255.0) as i32) == ((other.color.g * 255.0) as i32)
            && ((self.color.b * 255.0) as i32) == ((other.color.b * 255.0) as i32)
    }
}

impl Eq for MaterialGlow {}

impl std::hash::Hash for MaterialGlow {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.glow_type.hash(state);
        ((self.color.r * 255.0) as i32).hash(state);
        ((self.color.g * 255.0) as i32).hash(state);
        ((self.color.b * 255.0) as i32).hash(state);
    }
}

impl Default for MaterialGlow {
    fn default() -> Self {
        Self {
            glow_type: MaterialGlowType::Simple,
            color: Color3::WHITE,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, FromPrimitive)]
pub enum ModelCollisionShape {
    Sphere = 1,
    Aabb = 2,
    Oobb = 3,
    Mesh = 4,
}

impl Display for ModelCollisionShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ModelCollisionShape::Sphere => write!(f, "Sphere"),
            ModelCollisionShape::Aabb => write!(f, "AABB"),
            ModelCollisionShape::Oobb => write!(f, "OOBB"),
            ModelCollisionShape::Mesh => write!(f, "Mesh"),
        }
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
    pub struct ModelCollisionFlags: u32 {
        const NotMovable = 1 << 3;
        const NotPickable = 1 << 4;
        const HeightOnly = 1 << 5;
        const NotCameraCollision = 1 << 6;
        const Passthrough = 1 << 7;
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, FromPrimitive)]
enum ModelProperty {
    None = 0,
    Position = 1,
    Rotation = 2,
    Scale = 3,
    AxisRotation = 4, // unused
    BoneIndex = 5,
    DummyIndex = 6,
    Parent = 7,
    Animation = 8,
    Collision = 29,
    AnimationPath = 30,
    Range = 31,
    UseLightmap = 32,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, FromPrimitive)]
enum DummyAttachmentType {
    #[default]
    Normal = 0,
    DayNight = 1,
    LightContainer = 2,
}

impl RoseFile for ModelList {
    fn new() -> ModelList {
        Self::default()
    }

    fn read<R: ReadRoseExt>(&mut self, reader: &mut R) -> Result<(), RoseLibError> {
        let mesh_file_count = reader.read_u16()?;
        let mut meshes = Vec::with_capacity(mesh_file_count as usize);
        for _ in 0..mesh_file_count {
            meshes.push(reader.read_cstring()?);
        }

        let material_count = reader.read_u16()?;
        let mut materials = Vec::with_capacity(mesh_file_count as usize);
        for _ in 0..material_count {
            materials.push(ModelMaterial {
                path: reader.read_cstring()?,
                is_skin: reader.read_bool16()?,
                alpha_enabled: reader.read_bool16()?,
                two_sided: reader.read_bool16()?,
                alpha_test: {
                    let enabled = reader.read_bool16()?;
                    let alpha_ref = reader.read_u16()?;
                    if enabled {
                        Some(alpha_ref as u8)
                    } else {
                        None
                    }
                },
                z_test_enabled: reader.read_bool16()?,
                z_write_enabled: reader.read_bool16()?,
                blend_mode: {
                    let blend_mode = reader.read_u16()?;
                    if blend_mode > 0 {
                        FromPrimitive::from_u16(blend_mode)
                    } else {
                        None
                    }
                },
                specular_enabled: reader.read_bool16()?,
                alpha: reader.read_f32()?,
                glow: {
                    let glow_type = reader.read_u16()?;
                    let glow_color = reader.read_color3()?;
                    if glow_type > 0 {
                        FromPrimitive::from_u16(glow_type).map(|glow_type| MaterialGlow {
                            glow_type,
                            color: glow_color,
                        })
                    } else {
                        None
                    }
                },
            });
        }

        let effect_file_count = reader.read_u16()?;
        let mut effects = Vec::with_capacity(effect_file_count as usize);
        for _ in 0..effect_file_count {
            effects.push(reader.read_cstring()?);
        }

        let model_count = reader.read_u16()? as usize;
        self.models.reserve(model_count);
        for _ in 0..model_count {
            let mut model = Model::default();
            model.bounding_cylinder.radius = reader.read_u32()? as f32;
            model.bounding_cylinder.center = reader.read_vector2_i32()?;

            let part_count = reader.read_u16()?;
            if part_count == 0 {
                self.models.push(None);
                continue;
            }

            for _ in 0..part_count {
                let mesh_index = reader.read_i16()?;
                let material_index = reader.read_i16()?;
                let mut part = ModelPart {
                    mesh_path: meshes[mesh_index as usize].clone(),
                    material: if material_index >= 0 {
                        Some(materials[material_index as usize].clone())
                    } else {
                        None
                    },
                    ..Default::default()
                };

                loop {
                    let property_index = reader.read_u8()?;
                    let Some(property) = FromPrimitive::from_u8(property_index) else {
                        return Err(RoseLibError::Generic(format!(
                            "Invalid part property {}",
                            property_index
                        )));
                    };
                    if property == ModelProperty::None {
                        break;
                    }
                    let size = reader.read_u8()?;

                    match property {
                        ModelProperty::None => break,
                        ModelProperty::Position => part.position = reader.read_vector3_f32()?,
                        ModelProperty::Rotation => part.rotation = reader.read_quaternion_wxyz()?,
                        ModelProperty::Scale => part.scale = reader.read_vector3_f32()?,
                        ModelProperty::AxisRotation => {
                            let _unused = reader.read_quaternion_wxyz()?;
                        }
                        ModelProperty::BoneIndex => {
                            let bone_index = reader.read_i16()?;
                            if bone_index >= 0 {
                                part.bone_index = Some(bone_index as u16);
                            }
                        }
                        ModelProperty::DummyIndex => {
                            let dummy_index = reader.read_i16()?;
                            if dummy_index >= 0 {
                                part.dummy_index = Some(dummy_index as u16);
                            }
                        }
                        ModelProperty::Parent => {
                            let parent = reader.read_i16()?;
                            if parent > 0 {
                                part.parent = Some((parent - 1) as u16);
                            }
                        }
                        ModelProperty::Collision => {
                            let value = reader.read_u16()? as u32;
                            let shape = value & 0b111;
                            let flags = value & !0b111;
                            part.collision_shape = FromPrimitive::from_u32(shape);
                            part.collision_flags = ModelCollisionFlags::from_bits(flags).unwrap();
                        }
                        ModelProperty::AnimationPath => {
                            part.animation_path = Some(reader.read_string(size as u64)?);
                        }
                        ModelProperty::Range => {
                            let range_set_id = reader.read_i16()?;
                            if range_set_id > 0 {
                                part.range_set_id = Some(range_set_id as u16);
                            }
                        }
                        ModelProperty::UseLightmap => part.use_lightmap = reader.read_bool16()?,
                        ModelProperty::Animation => {
                            return Err(RoseLibError::Generic(
                                "Animation scene object property found but no handler.".to_string(),
                            ))
                        }
                    }
                }

                model.parts.push(part);
            }

            let dummy_point_count = reader.read_u16()?;
            for _ in 0..dummy_point_count {
                let effect_file_id = reader.read_i16()?;
                let attachment_type = reader.read_u16()?;
                let attachment = match FromPrimitive::from_u16(attachment_type) {
                    Some(DummyAttachmentType::Normal) => {
                        if effect_file_id >= 0 && !effects[effect_file_id as usize].is_empty() {
                            Some(ModelDummyAttachment::Effect {
                                path: effects[effect_file_id as usize].clone(),
                                only_visible_at_night: false,
                            })
                        } else {
                            None
                        }
                    }
                    Some(DummyAttachmentType::DayNight) => {
                        if effect_file_id >= 0 && !effects[effect_file_id as usize].is_empty() {
                            Some(ModelDummyAttachment::Effect {
                                path: effects[effect_file_id as usize].clone(),
                                only_visible_at_night: true,
                            })
                        } else {
                            None
                        }
                    }
                    Some(DummyAttachmentType::LightContainer) => {
                        if effect_file_id >= 0 && !effects[effect_file_id as usize].is_empty() {
                            Some(ModelDummyAttachment::Light {
                                name: effects[effect_file_id as usize].clone(),
                            })
                        } else {
                            None
                        }
                    }
                    None => None,
                };

                let mut dummy_point = ModelDummyPoint {
                    attachment,
                    ..Default::default()
                };

                loop {
                    let property_index = reader.read_u8()?;
                    let Some(property) = FromPrimitive::from_u8(property_index) else {
                        return Err(RoseLibError::Generic(format!(
                            "Invalid part property {}",
                            property_index
                        )));
                    };
                    if property == ModelProperty::None {
                        break;
                    }
                    let size = reader.read_u8()?;

                    match property {
                        ModelProperty::None => break,
                        ModelProperty::Position => {
                            dummy_point.position = reader.read_vector3_f32()?
                        }
                        ModelProperty::Rotation => {
                            dummy_point.rotation = reader.read_quaternion_wxyz()?
                        }
                        ModelProperty::Scale => dummy_point.scale = reader.read_vector3_f32()?,
                        ModelProperty::Parent => {
                            let parent = reader.read_i16()?;
                            if parent > 0 {
                                dummy_point.parent = Some((parent - 1) as u16);
                            }
                        }
                        _ => {
                            reader.seek(SeekFrom::Current(size as i64))?;
                        }
                    }
                }

                model.dummy_points.push(dummy_point);
            }

            model.bounding_box.min = reader.read_vector3_f32()?;
            model.bounding_box.max = reader.read_vector3_f32()?;

            self.models.push(Some(model));
        }

        Ok(())
    }

    fn write<W: WriteRoseExt>(&mut self, writer: &mut W) -> Result<(), RoseLibError> {
        let mut mesh_files = self
            .models
            .iter()
            .filter_map(|x| x.as_ref())
            // Deduplicate paths in a HashSet
            .flat_map(|model| model.parts.iter())
            .map(|part| part.mesh_path.clone())
            .collect::<HashSet<String>>()
            // Convert into a Vec
            .into_iter()
            .collect::<Vec<String>>();
        mesh_files.sort();

        // Collect into a map of path -> index for later use
        let mesh_files_map: HashMap<String, usize> = mesh_files
            .iter()
            .enumerate()
            .map(|(x, y)| (y.clone(), x))
            .collect();

        // Collect all materials
        let mut materials = self
            .models
            .iter()
            .filter_map(|x| x.as_ref())
            // Deduplicate paths in a HashSet
            .flat_map(|model| model.parts.iter())
            .filter_map(|part| part.material.clone())
            .collect::<HashSet<ModelMaterial>>()
            // Convert into a Vec
            .into_iter()
            .collect::<Vec<ModelMaterial>>();
        materials.sort_by(|lhs, rhs| lhs.path.cmp(&rhs.path));

        let materials_map: HashMap<ModelMaterial, usize> = materials
            .iter()
            .enumerate()
            .map(|(x, y)| (y.clone(), x))
            .collect();

        // Collect all effect files
        let mut effect_files = self
            .models
            .iter()
            .filter_map(|x| x.as_ref())
            // Deduplicate paths in a HashSet
            .flat_map(|model| model.dummy_points.iter())
            .filter_map(|dummy_point| dummy_point.attachment.as_ref())
            .map(|attachment| match attachment {
                ModelDummyAttachment::Effect { path, .. } => path.clone(),
                ModelDummyAttachment::Light { name } => name.clone(),
            })
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

        writer.write_u16(mesh_files.len() as u16)?;
        for mesh_path in mesh_files.iter() {
            writer.write_cstring(mesh_path)?;
        }

        writer.write_u16(materials.len() as u16)?;
        for mat in materials.iter() {
            writer.write_cstring(&mat.path)?;
            writer.write_bool16(mat.is_skin)?;
            writer.write_bool16(mat.alpha_enabled)?;
            writer.write_bool16(mat.two_sided)?;
            if let Some(alpha_ref) = mat.alpha_test {
                writer.write_bool16(true)?;
                writer.write_u16(alpha_ref as u16)?;
            } else {
                writer.write_bool16(false)?;
                writer.write_u16(0)?;
            }
            writer.write_bool16(mat.z_write_enabled)?;
            writer.write_bool16(mat.z_test_enabled)?;
            writer.write_u16(mat.blend_mode.map_or(0, |x| x as u16))?;
            writer.write_bool16(mat.specular_enabled)?;
            writer.write_f32(mat.alpha)?;
            match &mat.glow {
                Some(MaterialGlow { glow_type, color }) => {
                    writer.write_u16(*glow_type as u16)?;
                    writer.write_color3(color)?;
                }
                None => {
                    writer.write_u16(0)?;
                    writer.write_color3(&Color3::WHITE)?;
                }
            }
        }

        writer.write_u16(effect_files.len() as u16)?;
        for effect_path in effect_files.iter() {
            writer.write_cstring(effect_path)?;
        }

        writer.write_u16(self.models.len() as u16)?;
        for object in self.models.iter() {
            let Some(object) = object else {
                writer.write_u32(0)?;
                writer.write_vector2_i32(&Vector2::new(0, 0))?;
                writer.write_u16(0)?;
                continue;
            };

            writer.write_u32(object.bounding_cylinder.radius as u32)?;
            writer.write_vector2_i32(&object.bounding_cylinder.center)?;
            writer.write_u16(object.parts.len() as u16)?;
            if object.parts.is_empty() {
                continue;
            }

            for part in &object.parts {
                writer.write_u16(*mesh_files_map.get(&part.mesh_path).unwrap() as u16)?;
                if let Some(material) = &part.material {
                    writer.write_u16(*materials_map.get(material).unwrap() as u16)?;
                } else {
                    writer.write_u16(0)?;
                }

                if part.position.x != 0.0 || part.position.y != 0.0 || part.position.z != 0.0 {
                    writer.write_u8(ModelProperty::Position as u8)?;
                    writer.write_u8(12)?;
                    writer.write_vector3_f32(&part.position)?;
                }

                if !part.rotation.is_near_identity() {
                    writer.write_u8(ModelProperty::Rotation as u8)?;
                    writer.write_u8(16)?;
                    writer.write_quaternion_wxyz(&part.rotation)?;
                }

                if part.scale.x != 1.0 || part.scale.y != 1.0 || part.scale.z != 1.0 {
                    writer.write_u8(ModelProperty::Scale as u8)?;
                    writer.write_u8(12)?;
                    writer.write_vector3_f32(&part.scale)?;
                }

                if let Some(bone_index) = part.bone_index {
                    writer.write_u8(ModelProperty::BoneIndex as u8)?;
                    writer.write_u8(2)?;
                    writer.write_u16(bone_index)?;
                }

                if let Some(dummy_index) = part.dummy_index {
                    writer.write_u8(ModelProperty::DummyIndex as u8)?;
                    writer.write_u8(2)?;
                    writer.write_u16(dummy_index)?;
                }

                if let Some(parent) = part.parent {
                    writer.write_u8(ModelProperty::Parent as u8)?;
                    writer.write_u8(2)?;
                    writer.write_u16(parent + 1)?;
                }

                if let Some(collision_shape) = part.collision_shape {
                    writer.write_u8(ModelProperty::Collision as u8)?;
                    writer.write_u8(2)?;
                    let collision = collision_shape as u32 | part.collision_flags.bits();
                    writer.write_u16(collision as u16)?;
                }

                if let Some(path) = &part.animation_path {
                    writer.write_u8(ModelProperty::AnimationPath as u8)?;
                    writer.write_u8(path.len() as u8)?;
                    writer.write_string(path, path.len() as i32)?;
                }

                if let Some(range_set_id) = part.range_set_id {
                    writer.write_u8(ModelProperty::Range as u8)?;
                    writer.write_u8(2)?;
                    writer.write_u16(range_set_id)?;
                }

                if part.use_lightmap {
                    writer.write_u8(ModelProperty::UseLightmap as u8)?;
                    writer.write_u8(2)?;
                    writer.write_bool16(part.use_lightmap)?;
                }

                writer.write_u8(ModelProperty::None as u8)?;
            }

            writer.write_u16(object.dummy_points.len() as u16)?;
            for effect in &object.dummy_points {
                match &effect.attachment {
                    Some(ModelDummyAttachment::Effect {
                        path,
                        only_visible_at_night,
                    }) => {
                        writer.write_u16(*effect_files_map.get(path).unwrap() as u16)?;
                        if *only_visible_at_night {
                            writer.write_u16(DummyAttachmentType::DayNight as u16)?;
                        } else {
                            writer.write_u16(DummyAttachmentType::Normal as u16)?;
                        }
                    }
                    Some(ModelDummyAttachment::Light { name }) => {
                        writer.write_u16(*effect_files_map.get(name).unwrap() as u16)?;
                        writer.write_u16(DummyAttachmentType::LightContainer as u16)?;
                    }
                    None => {
                        writer.write_i16(-1)?;
                        writer.write_u16(0)?;
                    }
                }

                if effect.position.x != 0.0 || effect.position.y != 0.0 || effect.position.z != 0.0
                {
                    writer.write_u8(ModelProperty::Position as u8)?;
                    writer.write_u8(12)?;
                    writer.write_vector3_f32(&effect.position)?;
                }

                if !effect.rotation.is_near_identity() {
                    writer.write_u8(ModelProperty::Rotation as u8)?;
                    writer.write_u8(16)?;
                    writer.write_quaternion_wxyz(&effect.rotation)?;
                }

                if effect.scale.x != 1.0 || effect.scale.y != 1.0 || effect.scale.z != 1.0 {
                    writer.write_u8(ModelProperty::Scale as u8)?;
                    writer.write_u8(12)?;
                    writer.write_vector3_f32(&effect.scale)?;
                }

                if let Some(parent) = effect.parent {
                    writer.write_u8(ModelProperty::Parent as u8)?;
                    writer.write_u8(2)?;
                    writer.write_u16(parent + 1)?;
                }

                writer.write_u8(ModelProperty::None as u8)?;
            }

            writer.write_vector3_f32(&object.bounding_box.min)?;
            writer.write_vector3_f32(&object.bounding_box.max)?;
        }

        Ok(())
    }
}

impl RoseFile for Model {
    fn new() -> Model {
        Self::default()
    }

    fn read<R: ReadRoseExt>(&mut self, reader: &mut R) -> Result<(), RoseLibError> {
        enum ParseObject {
            Part {
                part: ModelPart,
                alpha_test: bool,
                alpha_ref: u8,
                glow_color: Color3,
            },
            Dummy {
                dummy: ModelDummyPoint,
                effect: String,
            },
        }
        let mut parse_object = None;

        let complete_object = |this: &mut Model, parse_object: &mut Option<ParseObject>| {
            if let Some(complete) = parse_object.take() {
                match complete {
                    ParseObject::Part {
                        mut part,
                        alpha_test,
                        alpha_ref,
                        glow_color,
                    } => {
                        if alpha_test {
                            if let Some(material) = part.material.as_mut() {
                                material.alpha_test = Some(alpha_ref);
                            }
                        }

                        if let Some(material) = part.material.as_mut() {
                            if let Some(glow) = material.glow.as_mut() {
                                glow.color = glow_color;
                            }
                        }

                        this.parts.push(part)
                    }
                    ParseObject::Dummy { mut dummy, effect } => {
                        match dummy.attachment.as_mut() {
                            Some(ModelDummyAttachment::Effect { path, .. }) => {
                                *path = effect;
                            }
                            Some(ModelDummyAttachment::Light { name }) => {
                                *name = effect;
                            }
                            None => {}
                        }
                        this.dummy_points.push(dummy);
                    }
                }
            }
        };

        fn parse_part_property(
            parse_object: Option<&mut ParseObject>,
            line_counter: i32,
            assign_fn: impl FnOnce(&mut ModelPart),
        ) -> Result<(), RoseLibError> {
            match parse_object {
                Some(ParseObject::Part { part, .. }) => {
                    assign_fn(part);
                    Ok(())
                }
                Some(ParseObject::Dummy { .. }) => Err(RoseLibError::Generic(format!(
                    "Unexpected property for a dummy point on line {}",
                    line_counter
                ))),
                None => Err(RoseLibError::Generic(format!(
                    "Unexpected property outside of an obj / point on line {}",
                    line_counter
                ))),
            }
        }

        fn parse_point_property(
            parse_object: Option<&mut ParseObject>,
            line_counter: i32,
            assign_fn: impl FnOnce(&mut ModelDummyPoint),
        ) -> Result<(), RoseLibError> {
            match parse_object {
                Some(ParseObject::Part { .. }) => Err(RoseLibError::Generic(format!(
                    "Unexpected property for a model part point on line {}",
                    line_counter
                ))),
                Some(ParseObject::Dummy { dummy, .. }) => {
                    assign_fn(dummy);
                    Ok(())
                }
                None => Err(RoseLibError::Generic(format!(
                    "Unexpected property outside of an obj / point on line {}",
                    line_counter
                ))),
            }
        }

        fn parse_part_material_property(
            parse_object: Option<&mut ParseObject>,
            line_counter: i32,
            assign_fn: impl FnOnce(&mut ModelMaterial),
        ) -> Result<(), RoseLibError> {
            parse_part_property(parse_object, line_counter, |part| {
                assign_fn(part.material.get_or_insert_with(ModelMaterial::default))
            })
        }

        pub fn parse<T: FromStr>(words: &[&str], idx: usize) -> Result<T, RoseLibError> {
            if idx >= words.len() {
                return Err(RoseLibError::Generic(format!(
                    "{} property missing required parameter",
                    words[0]
                )));
            }

            words[idx].parse::<T>().map_err(|_| {
                RoseLibError::Generic(format!(
                    "{} property has invalid value {}",
                    words[0], words[idx]
                ))
            })
        }

        let mut line = String::new();
        let mut line_counter = 0;
        loop {
            line.clear();
            if reader.read_line(&mut line)? == 0 {
                break;
            }
            line_counter += 1;

            let words: Vec<&str> = line.split_whitespace().collect();
            if words.is_empty() || words[0].starts_with("//") {
                continue;
            }

            if words[0] == "numobj" {
                let _unused_num_obj = parse::<i32>(&words, 1)?;
            } else if words[0] == "obj" {
                complete_object(self, &mut parse_object);
                parse_object = Some(ParseObject::Part {
                    part: ModelPart::default(),
                    alpha_ref: 128,
                    alpha_test: false,
                    glow_color: Color3::WHITE,
                });
            } else if words[0] == "numpoint" {
                let _unused_num_point = parse::<i32>(&words, 1)?;
            } else if words[0] == "point" {
                complete_object(self, &mut parse_object);
                parse_object = Some(ParseObject::Dummy {
                    dummy: ModelDummyPoint::default(),
                    effect: String::default(),
                });
            } else if words[0] == "cylinder" {
                self.bounding_cylinder.center.x = parse::<i32>(&words, 1)?;
                self.bounding_cylinder.center.y = parse::<i32>(&words, 2)?;
                self.bounding_cylinder.radius = parse::<f32>(&words, 3)?;
            } else if words[0] == "pos" {
                let position = Vector3 {
                    x: parse::<f32>(&words, 1)?,
                    y: parse::<f32>(&words, 2)?,
                    z: parse::<f32>(&words, 3)?,
                };
                match parse_object.as_mut() {
                    Some(ParseObject::Part { part, .. }) => {
                        part.position = position;
                    }
                    Some(ParseObject::Dummy { dummy, .. }) => {
                        dummy.position = position;
                    }
                    None => {
                        return Err(RoseLibError::Generic(format!(
                            "Unexpected {} outside of an obj / point on line {}",
                            words[0], line_counter
                        )))
                    }
                }
            } else if words[0] == "rot" {
                let rotation = Quaternion {
                    x: parse::<f32>(&words, 2)?,
                    y: parse::<f32>(&words, 3)?,
                    z: parse::<f32>(&words, 4)?,
                    w: parse::<f32>(&words, 1)?,
                };
                match parse_object.as_mut() {
                    Some(ParseObject::Part { part, .. }) => {
                        part.rotation = rotation;
                    }
                    Some(ParseObject::Dummy { dummy, .. }) => {
                        dummy.rotation = rotation;
                    }
                    None => {
                        return Err(RoseLibError::Generic(format!(
                            "Unexpected {} outside of an obj / point on line {}",
                            words[0], line_counter
                        )))
                    }
                }
            } else if words[0] == "scale" {
                let scale = Vector3 {
                    x: parse::<f32>(&words, 1)?,
                    y: parse::<f32>(&words, 2)?,
                    z: parse::<f32>(&words, 3)?,
                };
                match parse_object.as_mut() {
                    Some(ParseObject::Part { part, .. }) => {
                        part.scale = scale;
                    }
                    Some(ParseObject::Dummy { dummy, .. }) => {
                        dummy.scale = scale;
                    }
                    None => {
                        return Err(RoseLibError::Generic(format!(
                            "Unexpected {} outside of an obj / point on line {}",
                            words[0], line_counter
                        )))
                    }
                }
            } else if words[0] == "parent" {
                let parent = parse::<i32>(&words, 1)?;
                match parse_object.as_mut() {
                    Some(ParseObject::Part { part, .. }) => {
                        if parent > 0 {
                            part.parent = Some((parent - 1) as u16);
                        }
                    }
                    Some(ParseObject::Dummy { dummy, .. }) => {
                        if parent > 0 {
                            dummy.parent = Some((parent - 1) as u16);
                        }
                    }
                    None => {
                        return Err(RoseLibError::Generic(format!(
                            "Unexpected {} outside of an obj / point on line {}",
                            words[0], line_counter
                        )))
                    }
                }
            // Model Parts
            } else if words[0] == "mesh" {
                let path = words.get(1).unwrap_or(&"").to_string();
                if path.is_empty() {
                    return Err(RoseLibError::Generic("'mesh' property missing path".into()));
                }
                parse_part_property(parse_object.as_mut(), line_counter, |part| {
                    part.mesh_path = path
                })?;
            } else if words[0] == "mat" {
                let path = words.get(1).unwrap_or(&"").to_string();
                if path.is_empty() {
                    return Err(RoseLibError::Generic("'mat' property missing path".into()));
                }
                parse_part_material_property(parse_object.as_mut(), line_counter, |material| {
                    material.path = path
                })?;
            } else if words[0] == "isskin" {
                let is_skin = parse::<i32>(&words, 1)? != 0;
                parse_part_material_property(parse_object.as_mut(), line_counter, |material| {
                    material.is_skin = is_skin
                })?;
            } else if words[0] == "alpha" {
                let alpha_enabled = parse::<i32>(&words, 1)? != 0;
                parse_part_material_property(parse_object.as_mut(), line_counter, |material| {
                    material.alpha_enabled = alpha_enabled
                })?;
            } else if words[0] == "twoside" {
                let two_sided = parse::<i32>(&words, 1)? != 0;
                parse_part_material_property(parse_object.as_mut(), line_counter, |material| {
                    material.two_sided = two_sided
                })?;
            } else if words[0] == "alphatest" {
                match parse_object.as_mut() {
                    Some(ParseObject::Part { alpha_test, .. }) => {
                        *alpha_test = parse::<i32>(&words, 1)? != 0;
                    }
                    Some(ParseObject::Dummy { .. }) => {
                        return Err(RoseLibError::Generic(format!(
                            "Unexpected property for a dummy point on line {}",
                            line_counter
                        )))
                    }
                    None => {
                        return Err(RoseLibError::Generic(format!(
                            "Unexpected property outside of an obj / point on line {}",
                            line_counter
                        )))
                    }
                }
            } else if words[0] == "alpharef" {
                match parse_object.as_mut() {
                    Some(ParseObject::Part { alpha_ref, .. }) => {
                        *alpha_ref = parse::<u8>(&words, 1)?;
                    }
                    Some(ParseObject::Dummy { .. }) => {
                        return Err(RoseLibError::Generic(format!(
                            "Unexpected property for a dummy point on line {}",
                            line_counter
                        )))
                    }
                    None => {
                        return Err(RoseLibError::Generic(format!(
                            "Unexpected property outside of an obj / point on line {}",
                            line_counter
                        )))
                    }
                }
            } else if words[0] == "ztest" {
                let z_test_enabled = parse::<i32>(&words, 1)? != 0;
                parse_part_material_property(parse_object.as_mut(), line_counter, |material| {
                    material.z_test_enabled = z_test_enabled
                })?;
            } else if words[0] == "zwrite" {
                let z_write_enabled = parse::<i32>(&words, 1)? != 0;
                parse_part_material_property(parse_object.as_mut(), line_counter, |material| {
                    material.z_write_enabled = z_write_enabled
                })?;
            } else if words[0] == "blendtype" {
                let blend_mode_id = parse::<u32>(&words, 1)?;
                let blend_mode = FromPrimitive::from_u32(blend_mode_id);
                if blend_mode_id != 0 && blend_mode.is_none() {
                    return Err(RoseLibError::Generic(format!(
                        "Unexpected blendtype {} on line {}",
                        blend_mode_id, line_counter
                    )));
                }
                parse_part_material_property(parse_object.as_mut(), line_counter, |material| {
                    material.blend_mode = blend_mode
                })?;
            } else if words[0] == "specular" {
                let specular_enabled = parse::<i32>(&words, 1)? != 0;
                parse_part_material_property(parse_object.as_mut(), line_counter, |material| {
                    material.specular_enabled = specular_enabled
                })?;
            } else if words[0] == "alphavalue" {
                let alpha = parse::<f32>(&words, 1)?;
                parse_part_material_property(parse_object.as_mut(), line_counter, |material| {
                    material.alpha = alpha
                })?;
            } else if words[0] == "glowtype" {
                let glow_type_id = parse::<u32>(&words, 1)?;
                let glow_type = FromPrimitive::from_u32(glow_type_id);
                if glow_type_id != 0 && glow_type.is_none() {
                    return Err(RoseLibError::Generic(format!(
                        "Unexpected glowtype {} on line {}",
                        glow_type_id, line_counter
                    )));
                }
                if let Some(glow_type) = glow_type {
                    parse_part_material_property(
                        parse_object.as_mut(),
                        line_counter,
                        |material| {
                            material.glow = Some(MaterialGlow {
                                glow_type,
                                color: Color3::WHITE,
                            })
                        },
                    )?;
                }
            } else if words[0] == "glowcolor" {
                match parse_object.as_mut() {
                    Some(ParseObject::Part { glow_color, .. }) => {
                        *glow_color = Color3 {
                            r: parse::<f32>(&words, 1)?,
                            g: parse::<f32>(&words, 2)?,
                            b: parse::<f32>(&words, 3)?,
                        };
                    }
                    Some(ParseObject::Dummy { .. }) => {
                        return Err(RoseLibError::Generic(format!(
                            "Unexpected property for a dummy point on line {}",
                            line_counter
                        )))
                    }
                    None => {
                        return Err(RoseLibError::Generic(format!(
                            "Unexpected property outside of an obj / point on line {}",
                            line_counter
                        )))
                    }
                }
            } else if words[0] == "linkdummy" {
                let dummy_index = parse::<i32>(&words, 1)?;
                if dummy_index >= 0 {
                    parse_part_property(parse_object.as_mut(), line_counter, |part| {
                        part.dummy_index = Some(dummy_index as u16)
                    })?;
                }
            } else if words[0] == "bonenumber" {
                let bone_index = parse::<i32>(&words, 1)?;
                if bone_index >= 0 {
                    parse_part_property(parse_object.as_mut(), line_counter, |part| {
                        part.bone_index = Some(bone_index as u16);
                    })?;
                }
            } else if words[0] == "collision" {
                let value = parse::<u32>(&words, 1)?;
                let shape = value & 0b111;
                let flags = value & !0b111;
                let collision_shape = FromPrimitive::from_u32(shape);
                let collision_flags = ModelCollisionFlags::from_bits(flags).unwrap();
                if let Some(collision_shape) = collision_shape {
                    parse_part_property(parse_object.as_mut(), line_counter, |part| {
                        part.collision_shape = Some(collision_shape);
                        part.collision_flags = collision_flags;
                    })?;
                }
            } else if words[0] == "anim" {
                let path = words.get(1).map(|s| s.to_string());
                if let Some(ref path) = path {
                    if path.is_empty() {
                        return Err(RoseLibError::Generic("'anim' property missing path".into()));
                    }
                }
                parse_part_property(parse_object.as_mut(), line_counter, |part| {
                    part.animation_path = path;
                })?;
            } else if words[0] == "rangeset" {
                let range_set_id = parse::<i32>(&words, 1)?;
                if range_set_id > 0 {
                    parse_part_property(parse_object.as_mut(), line_counter, |part| {
                        part.range_set_id = Some(range_set_id as u16);
                    })?;
                }
            } else if words[0] == "uselightmap" {
                let use_lightmap = parse::<i32>(&words, 1)? != 0;
                parse_part_property(parse_object.as_mut(), line_counter, |part| {
                    part.use_lightmap = use_lightmap;
                })?;
            // Dummy points
            } else if words[0] == "effect" {
                match parse_object.as_mut() {
                    Some(ParseObject::Dummy { effect, .. }) => {
                        if let Some(word) = words.get(1) {
                            // This is optional for some reason
                            *effect = word.to_string();
                        }
                    }
                    Some(ParseObject::Part { .. }) => {
                        return Err(RoseLibError::Generic(format!(
                            "Unexpected property for a model part on line {}",
                            line_counter
                        )))
                    }
                    None => {
                        return Err(RoseLibError::Generic(format!(
                            "Unexpected property outside of an obj / point on line {}",
                            line_counter
                        )))
                    }
                }
            } else if words[0] == "type" {
                let attachment_type_id = parse::<u32>(&words, 1)?;
                let attachment_type = FromPrimitive::from_u32(attachment_type_id);

                if let Some(attachment_type) = attachment_type {
                    parse_point_property(parse_object.as_mut(), line_counter, |dummy| {
                        match attachment_type {
                            DummyAttachmentType::Normal => {
                                dummy.attachment = Some(ModelDummyAttachment::Effect {
                                    path: String::default(),
                                    only_visible_at_night: false,
                                });
                            }
                            DummyAttachmentType::DayNight => {
                                dummy.attachment = Some(ModelDummyAttachment::Effect {
                                    path: String::default(),
                                    only_visible_at_night: true,
                                });
                            }
                            DummyAttachmentType::LightContainer => {
                                dummy.attachment = Some(ModelDummyAttachment::Light {
                                    name: String::default(),
                                });
                            }
                        }
                    })?;
                }
            }
        }
        complete_object(self, &mut parse_object);

        Ok(())
    }

    fn write<W: WriteRoseExt>(&mut self, writer: &mut W) -> Result<(), RoseLibError> {
        writeln!(writer, "numobj {}", self.parts.len())?;

        if self.bounding_cylinder.radius != 0.0
            || self.bounding_cylinder.center.x != 0
            || self.bounding_cylinder.center.y != 0
        {
            writeln!(
                writer,
                "cylinder {} {} {}",
                self.bounding_cylinder.center.x,
                self.bounding_cylinder.center.y,
                self.bounding_cylinder.radius
            )?;
        }

        for (part_index, part) in self.parts.iter().enumerate() {
            writeln!(writer)?;
            writeln!(writer, "\tpart {}", part_index + 1)?;
            writeln!(writer, "\tmesh {}", &part.mesh_path)?;

            if part.position.x != 0.0 || part.position.y != 0.0 || part.position.z != 0.0 {
                writeln!(
                    writer,
                    "\tpos {} {} {}",
                    part.position.x, part.position.y, part.position.z
                )?;
            }

            if !part.rotation.is_near_identity() {
                writeln!(
                    writer,
                    "\trot {} {} {} {}",
                    part.rotation.w, part.rotation.x, part.rotation.y, part.rotation.z
                )?;
            }

            if part.scale.x != 0.0 || part.scale.y != 0.0 || part.scale.z != 0.0 {
                writeln!(
                    writer,
                    "\tscale {} {} {}",
                    part.scale.x, part.scale.y, part.scale.z
                )?;
            }

            if let Some(collision_shape) = part.collision_shape {
                writeln!(
                    writer,
                    "\tcollision {}",
                    collision_shape as u32 | part.collision_flags.bits()
                )?;
            }

            if let Some(animation_path) = &part.animation_path {
                writeln!(writer, "\tanim {}", animation_path)?;
            }

            if let Some(parent) = part.parent {
                writeln!(writer, "\tparent {}", parent + 1)?;
            }

            if let Some(dummy_index) = part.dummy_index {
                writeln!(writer, "\tlinkdummy {}", dummy_index)?;
            }

            if let Some(bone_index) = part.bone_index {
                writeln!(writer, "\tbonenumber {}", bone_index)?;
            }

            if let Some(range_set_id) = part.range_set_id {
                writeln!(writer, "\trangeset {}", range_set_id)?;
            }

            if part.use_lightmap {
                writeln!(writer, "\tuselightmap {}", part.use_lightmap as i32)?;
            }

            if let Some(material) = &part.material {
                writeln!(writer, "\tmat {}", &material.path)?;

                if material.is_skin {
                    writeln!(writer, "\t\tisskin 1")?;
                }

                if material.alpha != 0.0 {
                    writeln!(writer, "\t\talpha {}", material.alpha)?;
                }

                if material.two_sided {
                    writeln!(writer, "\t\ttwoside 1")?;
                }

                if let Some(alpha_ref) = material.alpha_test {
                    writeln!(writer, "\t\talphatest 1")?;
                    writeln!(writer, "\t\talpharef {}", alpha_ref)?;
                }

                if !material.z_test_enabled {
                    writeln!(writer, "\t\tztest 0")?;
                }

                if !material.z_write_enabled {
                    writeln!(writer, "\t\tzwrite 0")?;
                }

                if let Some(blend_mode) = &material.blend_mode {
                    writeln!(writer, "\t\tblendtype {}", *blend_mode as i32)?;
                }

                if material.specular_enabled {
                    writeln!(writer, "\t\tspecular 1")?;
                }

                if material.alpha != 1.0 {
                    writeln!(writer, "\t\talpha {}", material.alpha)?;
                }

                if let Some(glow) = &material.glow {
                    writeln!(writer, "\t\tglowtype {}", glow.glow_type as i32)?;
                    writeln!(
                        writer,
                        "\t\tglowcolor {} {} {}",
                        glow.color.r, glow.color.g, glow.color.b
                    )?;
                }
            }
        }

        if !self.dummy_points.is_empty() {
            writeln!(writer)?;
            writeln!(writer, "numpoints {}", self.dummy_points.len())?;
        }

        for (index, dummy_point) in self.dummy_points.iter().enumerate() {
            writeln!(writer)?;
            writeln!(writer, "\tpoint {}", index + 1)?;

            match &dummy_point.attachment {
                Some(ModelDummyAttachment::Effect {
                    path,
                    only_visible_at_night,
                }) => {
                    writeln!(writer, "\teffect {}", path)?;
                    if *only_visible_at_night {
                        writeln!(writer, "\ttype {}", DummyAttachmentType::DayNight as i32)?;
                    } else {
                        writeln!(writer, "\ttype {}", DummyAttachmentType::Normal as i32)?;
                    }
                }
                Some(ModelDummyAttachment::Light { name }) => {
                    writeln!(writer, "\teffect {}", name)?;
                    writeln!(
                        writer,
                        "\ttype {}",
                        DummyAttachmentType::LightContainer as i32
                    )?;
                }
                None => {
                    writeln!(writer, "\teffect ")?;
                    writeln!(writer, "\ttype 0")?;
                }
            }

            if dummy_point.position.x != 0.0
                || dummy_point.position.y != 0.0
                || dummy_point.position.z != 0.0
            {
                writeln!(
                    writer,
                    "\tpos {} {} {}",
                    dummy_point.position.x, dummy_point.position.y, dummy_point.position.z
                )?;
            }

            if !dummy_point.rotation.is_near_identity() {
                writeln!(
                    writer,
                    "\trot {} {} {} {}",
                    dummy_point.rotation.w,
                    dummy_point.rotation.x,
                    dummy_point.rotation.y,
                    dummy_point.rotation.z
                )?;
            }

            if dummy_point.scale.x != 0.0
                || dummy_point.scale.y != 0.0
                || dummy_point.scale.z != 0.0
            {
                writeln!(
                    writer,
                    "\tscale {} {} {}",
                    dummy_point.scale.x, dummy_point.scale.y, dummy_point.scale.z
                )?;
            }

            if let Some(parent) = dummy_point.parent {
                writeln!(writer, "\tparent {}", parent + 1)?;
            }
        }

        Ok(())
    }
}
