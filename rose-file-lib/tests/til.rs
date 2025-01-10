use std::path::PathBuf;

use rose_file_lib::files::TIL;
use rose_file_lib::io::RoseFile;

#[test]
fn read_til() {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.push("tests");
    root.push("data");

    let file = root.join("31_30.til");
    let til = TIL::from_path(&file).unwrap();

    assert_eq!(til.tiles.len(), 16);
    for t in til.tiles {
        assert_eq!(t.len(), 16);
    }
}
