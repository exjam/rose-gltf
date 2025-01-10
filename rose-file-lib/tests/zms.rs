use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;

use rose_file_lib::files::zms::MeshType;
use rose_file_lib::files::ZMS;
use rose_file_lib::io::RoseFile;

use pretty_assertions::assert_eq;

#[test]
#[allow(clippy::bool_assert_comparison)]
fn read_zms() {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.push("tests");
    root.push("data");

    let file1 = root.join("headbad01.zms");
    let file2 = root.join("stone014.zms");
    let file3 = root.join("cart01_ability01.zms");
    let file4 = root.join("mob_wolf2.zms");

    let model1 = ZMS::from_path(&file1).unwrap();
    assert_eq!(model1.identifier.as_str(), "ZMS0008");
    assert_eq!(model1.format, 182);
    assert_eq!(model1.positions_enabled(), true);
    assert_eq!(model1.normals_enabled(), true);
    assert_eq!(model1.colors_enabled(), false);
    assert_eq!(model1.bones_enabled(), true);
    assert_eq!(model1.tangents_enabled(), false);
    assert_eq!(model1.uv1_enabled(), true);
    assert_eq!(model1.uv2_enabled(), false);
    assert_eq!(model1.uv3_enabled(), false);
    assert_eq!(model1.uv4_enabled(), false);

    assert_eq!(model1.bones.len(), 8);
    assert_eq!(model1.vertices.len(), 336);
    assert_eq!(model1.indices.len(), 578);
    assert_eq!(model1.materials.len(), 6);
    assert_eq!(model1.strips.len(), 0);
    assert_eq!(model1.mesh_type, Some(MeshType::Static));

    let model2 = ZMS::from_path(&file2).unwrap();
    assert_eq!(model2.identifier.as_str(), "ZMS0007");
    assert_eq!(model2.format, 390);
    assert_eq!(model2.positions_enabled(), true);
    assert_eq!(model2.normals_enabled(), true);
    assert_eq!(model2.colors_enabled(), false);
    assert_eq!(model2.bones_enabled(), false);
    assert_eq!(model2.tangents_enabled(), false);
    assert_eq!(model2.uv1_enabled(), true);
    assert_eq!(model2.uv2_enabled(), true);
    assert_eq!(model2.uv3_enabled(), false);
    assert_eq!(model2.uv4_enabled(), false);

    assert_eq!(model2.bones.len(), 0);
    assert_eq!(model2.vertices.len(), 131);
    assert_eq!(model2.indices.len(), 128);
    assert_eq!(model2.materials.len(), 0);
    assert_eq!(model2.strips.len(), 0);
    assert_eq!(model2.mesh_type, None);

    let model3 = ZMS::from_path(&file3).unwrap();
    assert_eq!(model3.identifier.as_str(), "ZMS0008");
    assert_eq!(model3.format, 134);
    assert_eq!(model3.positions_enabled(), true);
    assert_eq!(model3.normals_enabled(), true);
    assert_eq!(model3.colors_enabled(), false);
    assert_eq!(model3.bones_enabled(), false);
    assert_eq!(model3.tangents_enabled(), false);
    assert_eq!(model3.uv1_enabled(), true);
    assert_eq!(model3.uv2_enabled(), false);
    assert_eq!(model3.uv3_enabled(), false);
    assert_eq!(model3.uv4_enabled(), false);

    assert_eq!(model3.bones.len(), 0);
    assert_eq!(model3.vertices.len(), 544);
    assert_eq!(model3.indices.len(), 532);
    assert_eq!(model3.materials.len(), 2);
    assert_eq!(model3.strips.len(), 0);
    assert_eq!(model3.mesh_type, Some(MeshType::Static));

    let model4 = ZMS::from_path(&file4).unwrap();
    assert_eq!(model4.identifier.as_str(), "ZMS0006");
    assert_eq!(model4.format, 134);
    assert_eq!(model4.positions_enabled(), true);
    assert_eq!(model4.normals_enabled(), true);
    assert_eq!(model4.colors_enabled(), false);
    assert_eq!(model4.bones_enabled(), false);
    assert_eq!(model4.tangents_enabled(), false);
    assert_eq!(model4.uv1_enabled(), true);
    assert_eq!(model4.uv2_enabled(), false);
    assert_eq!(model4.uv3_enabled(), false);
    assert_eq!(model4.uv4_enabled(), false);

    assert_eq!(model4.bones.len(), 0);
    assert_eq!(model4.vertices.len(), 76);
    assert_eq!(model4.indices.len(), 84);
    assert_eq!(model4.materials.len(), 5);
    assert_eq!(model4.strips.len(), 0);
    assert_eq!(model4.mesh_type, None);
}

#[test]
fn write_zms() {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.push("tests");
    root.push("data");

    let file1 = root.join("headbad01.zms");
    let file2 = root.join("stone014.zms");
    let file3 = root.join("cart01_ability01.zms");

    for zms_file in [file1, file2, file3].iter() {
        let f = File::open(zms_file).unwrap();
        let zms_size = f.metadata().unwrap().len();

        let mut orig_zms = ZMS::from_path(zms_file).unwrap();

        let buffer = vec![0u8; zms_size as usize];

        let mut cursor = Cursor::new(buffer);
        orig_zms.write(&mut cursor).unwrap();

        cursor.set_position(0);

        let mut new_zms = ZMS::new();
        new_zms.read(&mut cursor).unwrap();

        if orig_zms.identifier.as_str() == "ZMS0007" {
            orig_zms.identifier = String::from("ZMS0008");
        }

        assert_eq!(orig_zms, new_zms);
    }
}
