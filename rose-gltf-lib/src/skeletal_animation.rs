use bytes::{BufMut, BytesMut};
use glam::{Mat4, Quat, Vec3};
use rose_file_lib::files::{zmd::Bone, ZMD, ZMO};

use gltf_json::{
    accessor, buffer,
    scene::UnitQuaternion,
    validation::{Checked, USize64},
    Index, Node, Skin,
};

use crate::{
    animation::{load_animation, GetAnimationChannelNode},
    pad_align,
};

fn transform_children(zmd: &ZMD, bone_transforms: &mut Vec<Mat4>, bone_index: usize) {
    for (child_id, child_bone) in zmd.bones.iter().enumerate() {
        if child_id == bone_index || child_bone.parent as usize != bone_index {
            continue;
        }

        bone_transforms[child_id] = bone_transforms[bone_index] * bone_transforms[child_id];
        transform_children(zmd, bone_transforms, child_id);
    }
}

fn bone_to_node(bone: &Bone) -> (Node, glam::Mat4) {
    let translation = Vec3::new(bone.position.x, bone.position.z, -bone.position.y) / 100.0;
    let rotation = Quat::from_xyzw(
        bone.rotation.x,
        bone.rotation.z,
        -bone.rotation.y,
        bone.rotation.w,
    )
    .normalize();

    let node = Node {
        name: Some(bone.name.clone()),
        camera: None,
        children: None,
        extensions: Default::default(),
        extras: Default::default(),
        matrix: None,
        mesh: None,
        rotation: Some(UnitQuaternion([
            rotation.x, rotation.y, rotation.z, rotation.w,
        ])),
        scale: None,
        translation: Some([translation.x, translation.y, translation.z]),
        skin: None,
        weights: None,
    };
    let bind_pose = glam::Mat4::from_rotation_translation(rotation, translation);

    (node, bind_pose)
}

pub fn load_skeleton(
    root: &mut gltf_json::Root,
    binary_data: &mut BytesMut,
    name: &str,
    zmd: &ZMD,
) -> Index<Skin> {
    let bone_node_index_start = root.nodes.len();
    let mut joints = Vec::new();
    let mut bind_poses = Vec::new();

    pad_align(binary_data);

    // Add root node to scene
    root.scenes[0]
        .nodes
        .push(Index::new(root.nodes.len() as u32));

    // Create nodes for each bone
    for (bone_index, bone) in zmd.bones.iter().enumerate() {
        let (node, bind_pose) = bone_to_node(bone);

        root.nodes.push(node);
        joints.push(Index::new(bone_node_index_start as u32 + bone_index as u32));
        bind_poses.push(bind_pose);
    }

    // Create nodes for each dummy bone
    for (dummy_bone_index, dummy_bone) in zmd.dummy_bones.iter().enumerate() {
        let (mut node, _bind_pose) = bone_to_node(dummy_bone);
        if !dummy_bone.name.is_empty() {
            node.name = Some(format!("dummy_{}_{}", dummy_bone_index, &dummy_bone.name));
        } else {
            node.name = Some(format!("dummy_{}", dummy_bone_index));
        }

        root.nodes.push(node);
    }

    // Assign parents
    for (bone_index, bone) in zmd.bones.iter().chain(&zmd.dummy_bones).enumerate() {
        if bone_index == bone.parent as usize {
            continue;
        }

        let parent_node_index = bone_node_index_start + bone.parent as usize;
        let parent_node = &mut root.nodes[parent_node_index];

        let node_index = bone_node_index_start + bone_index;
        parent_node
            .children
            .get_or_insert_with(Vec::new)
            .push(Index::new(node_index as u32));
    }

    // Calculate inverse bind pose
    transform_children(zmd, &mut bind_poses, 0);
    let inverse_bind_pose: Vec<Mat4> = bind_poses.iter().map(|x| x.inverse()).collect();

    let skeleton_data_start = binary_data.len();
    for mtx in inverse_bind_pose.iter() {
        for f in mtx.to_cols_array() {
            binary_data.put_f32_le(f);
        }
    }
    let skeleton_data_length = binary_data.len() - skeleton_data_start;

    let buffer_view_index = root.buffer_views.len() as u32;
    root.buffer_views.push(buffer::View {
        name: Some(format!("{}_SkeletonBufferView", name)),
        buffer: Index::new(0),
        byte_length: USize64::from(skeleton_data_length),
        byte_offset: Some(USize64::from(skeleton_data_start)),
        byte_stride: None,
        extensions: Default::default(),
        extras: Default::default(),
        target: None,
    });

    let accessor_index = root.accessors.len() as u32;
    root.accessors.push(accessor::Accessor {
        name: Some(format!("{}_SkeletonAccessor", name)),
        buffer_view: Some(Index::new(buffer_view_index)),
        byte_offset: Some(USize64(0)),
        count: USize64::from(inverse_bind_pose.len()),
        component_type: Checked::Valid(accessor::GenericComponentType(
            accessor::ComponentType::F32,
        )),
        extensions: Default::default(),
        extras: Default::default(),
        type_: Checked::Valid(accessor::Type::Mat4),
        min: None,
        max: None,
        normalized: false,
        sparse: None,
    });

    let skin_index = root.skins.len() as u32;
    root.skins.push(Skin {
        name: Some(name.to_string()),
        extensions: Default::default(),
        extras: Default::default(),
        inverse_bind_matrices: Some(Index::new(accessor_index)),
        skeleton: Some(joints[0]),
        joints,
    });
    Index::new(skin_index)
}

impl GetAnimationChannelNode for Index<Skin> {
    fn get(&self, root: &mut gltf_json::Root, channel: u32) -> Index<Node> {
        root.get(*self).unwrap().joints[channel as usize]
    }
}

pub fn load_skeletal_animation(
    root: &mut gltf_json::Root,
    binary_data: &mut BytesMut,
    name: &str,
    skin_index: Index<Skin>,
    zmo: &ZMO,
) {
    load_animation(root, binary_data, zmo, name, skin_index)
}
