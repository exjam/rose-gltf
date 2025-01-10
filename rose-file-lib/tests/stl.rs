use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;

use rose_file_lib::files::stl::StringTableType;
use rose_file_lib::files::STL;
use rose_file_lib::io::RoseFile;

#[test]
fn read_stl() {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.push("tests");
    root.push("data");

    let stl_data = [
        ("list_faceitem_s.stl", StringTableType::Description, 80),
        ("list_quest_s.stl", StringTableType::Quest, 243),
        ("str_itemtype.stl", StringTableType::Text, 156),
    ];

    for (filename, format, row_count) in &stl_data {
        let file = root.join(filename);
        let stl = STL::from_path(&file).unwrap();

        assert_eq!(stl.entry_type, *format);
        assert_eq!(stl.entries.len(), *row_count as usize);
    }
}

#[test]
fn write_stl() {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.push("tests");
    root.push("data");

    let stl_data = [
        "list_faceitem_s.stl",
        "list_quest_s.stl",
        "str_itemtype.stl",
    ];

    for filename in &stl_data {
        let stl_file = root.join(filename);

        let f = File::open(&stl_file).unwrap();
        let stl_size = f.metadata().unwrap().len();

        let mut orig_stl = STL::from_path(&stl_file).unwrap();
        let buffer = vec![0u8; stl_size as usize];

        let mut cursor = Cursor::new(buffer);
        orig_stl.write(&mut cursor).unwrap();
        cursor.set_position(0);

        let mut new_stl = STL::new();
        new_stl.read(&mut cursor).unwrap();

        assert_eq!(orig_stl, new_stl);
    }
}
