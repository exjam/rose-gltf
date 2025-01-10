use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;

use rose_file_lib::files::ZSC;
use rose_file_lib::io::RoseFile;

#[test]
#[allow(clippy::bool_assert_comparison)]
fn read_zsc() {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.push("tests");
    root.push("data");

    let file = root.join("part_npc.zsc");
    let zsc = ZSC::from_path(&file).unwrap();

    assert_eq!(zsc.models.len(), 697);

    assert_eq!(zsc.models[1].as_ref().unwrap().parts.len(), 1);
    assert_eq!(
        zsc.models[1].as_ref().unwrap().parts[0].mesh_path,
        r"3DData\NPC\animal\larva\larva1.ZMS"
    );
    let Some(material) = &zsc.models[1].as_ref().unwrap().parts[0].material else {
        panic!("Unexpected material value");
    };
    assert_eq!(material.path, r"3DData\NPC\animal\larva\larva1.dds");
    assert_eq!(material.is_skin, true);
    assert_eq!(material.alpha_enabled, false);
    assert_eq!(material.two_sided, false);
    assert_eq!(material.alpha_test, Some(128));
    assert_eq!(material.z_write_enabled, true);
    assert_eq!(material.z_test_enabled, true);
    assert_eq!(material.blend_mode, None);
    assert_eq!(material.specular_enabled, false);
    assert_eq!(material.alpha, 1.0);
    assert_eq!(material.glow, None);

    // Test file with weird effect values
    let file2 = root.join("list_weapon.zsc");
    let zsc2 = ZSC::from_path(&file2).unwrap();

    assert_eq!(zsc.models[1].as_ref().unwrap().parts.len(), 1);
    assert_eq!(
        zsc2.models[1].as_ref().unwrap().parts[0].mesh_path,
        r#"3Ddata/WEAPON/WEAPON/onehand/osw15/osw15.zms"#
    );

    let Some(material) = &zsc2.models[1].as_ref().unwrap().parts[0].material else {
        panic!("Unexpected material value");
    };
    assert_eq!(
        material.path,
        r#"3Ddata/WEAPON/WEAPON/onehand/osw15/osw15.dds"#
    );
    assert_eq!(material.is_skin, false);
    assert_eq!(material.alpha_enabled, false);
    assert_eq!(material.two_sided, false);
    assert_eq!(material.alpha_test, Some(128));
    assert_eq!(material.z_write_enabled, true);
    assert_eq!(material.z_test_enabled, true);
    assert_eq!(material.blend_mode, None);
    assert_eq!(material.specular_enabled, false);
    assert_eq!(material.alpha, 1.0);
    assert_eq!(material.glow, None);
}

#[test]
fn write_zsc() {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.push("tests");
    root.push("data");

    let zsc_file = root.join("part_npc.zsc");

    let f = File::open(&zsc_file).unwrap();
    let zsc_size = f.metadata().unwrap().len();

    let mut orig_zsc = ZSC::from_path(&zsc_file).unwrap();

    let buffer: Vec<u8> = vec![0u8; zsc_size as usize];

    let mut cursor = Cursor::new(buffer);
    orig_zsc.write(&mut cursor).unwrap();

    cursor.set_position(0);

    let mut new_zsc = ZSC::new();
    new_zsc.read(&mut cursor).unwrap();

    assert_eq!(orig_zsc.models.len(), new_zsc.models.len());
    for (orig, new) in orig_zsc.models.iter().zip(new_zsc.models.iter()) {
        assert_eq!(orig, new);
    }
}
