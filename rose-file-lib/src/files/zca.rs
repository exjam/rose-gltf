use serde::{Deserialize, Serialize};

use crate::error::RoseLibError;
use crate::io::{ReadRoseExt, RoseFile, WriteRoseExt};
use crate::utils::Vector3;

/// Camera File
pub type ZCA = Camera;

#[derive(Default, Serialize, Deserialize)]
enum ProjectionType {
    #[default]
    Orthographic,
    Perspective,
}

impl TryFrom<u32> for ProjectionType {
    type Error = RoseLibError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ProjectionType::Orthographic),
            1 => Ok(ProjectionType::Perspective),
            _ => Err(RoseLibError::Generic(format!(
                "Invalid Camera Type: {}",
                value
            ))),
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Camera {
    projection_type: ProjectionType,
    model_view: [f32; 16],
    projection: [f32; 16],
    field_of_view: f32,
    aspect_ratio: f32,
    near_plane: f32,
    far_plane: f32,
    eye_pos: Vector3<f32>,
    eye_center: Vector3<f32>,
    up: Vector3<f32>,
}

impl RoseFile for Camera {
    fn new() -> Camera {
        Self::default()
    }

    fn read<R: ReadRoseExt>(&mut self, reader: &mut R) -> Result<(), RoseLibError> {
        let identifier = reader.read_string(7)?;

        if identifier != "ZCA0001" {
            return Err(RoseLibError::Generic(format!(
                "Unrecognized ZCA identifier: {}",
                identifier
            )));
        }

        self.projection_type = ProjectionType::try_from(reader.read_u32()?)?;

        for i in 0..16 {
            self.model_view[i] = reader.read_f32()?;
        }

        for i in 0..16 {
            self.projection[i] = reader.read_f32()?;
        }

        self.field_of_view = reader.read_f32()?;
        self.aspect_ratio = reader.read_f32()?;
        self.near_plane = reader.read_f32()?;
        self.far_plane = reader.read_f32()?;

        self.eye_pos = reader.read_vector3_f32()?;
        self.eye_center = reader.read_vector3_f32()?;
        self.up = reader.read_vector3_f32()?;

        Ok(())
    }

    fn write<W: WriteRoseExt>(&mut self, _writer: &mut W) -> Result<(), RoseLibError> {
        unimplemented!();
    }
}
