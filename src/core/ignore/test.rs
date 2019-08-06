use super::*;

use std::path::PathBuf;

#[test]
fn test_entry_path_new_sucess_with_absolute_path_win() {
    let root = PathBuf::from("\\\\?\\C:\\Users\\test\\Documents\\important");
    let entry =
        PathBuf::from("\\\\?\\C:\\Users\\test\\Documents\\important\\some_dir\\an_file.txt");

    let ep = EntryPath::new(&root, &entry).unwrap();

    assert_eq!(
        ep.parts(),
        &["some_dir".to_owned(), "an_file.txt".to_owned()],
    );
}

#[test]
fn test_entry_path_new_sucess_with_absolute_path_unix() {
    let root = PathBuf::from("/home/test/Documents/important");
    let entry = PathBuf::from("/home/test/Documents/important/some_dir/an_file.txt");

    let ep = EntryPath::new(&root, &entry).unwrap();

    assert_eq!(
        ep.parts(),
        &["some_dir".to_owned(), "an_file.txt".to_owned()],
    );
}

#[test]
fn test_entry_path_new_sucess_with_relative_path_win() {
    let root = PathBuf::from("important");
    let entry = PathBuf::from("important\\some_dir\\an_file.txt");

    let ep = EntryPath::new(&root, &entry).unwrap();

    assert_eq!(
        ep.parts(),
        &["some_dir".to_owned(), "an_file.txt".to_owned()],
    );
}

#[test]
fn test_entry_path_new_sucess_with_relative_path_unix() {
    let root = PathBuf::from("important");
    let entry = PathBuf::from("important/some_dir/an_file.txt");

    let ep = EntryPath::new(&root, &entry).unwrap();

    assert_eq!(
        ep.parts(),
        &["some_dir".to_owned(), "an_file.txt".to_owned()],
    );
}

#[test]
fn test_entry_path_new_sucess_with_root_win() {
    let root = PathBuf::from("\\\\?\\C:\\Users\\test\\Documents\\important");
    let entry = PathBuf::from("\\\\?\\C:\\Users\\test\\Documents\\important");

    let ep = EntryPath::new(&root, &entry).unwrap();

    assert_eq!(ep.parts().len(), 0);
}

#[test]
fn test_entry_path_new_sucess_with_root_unix() {
    let root = PathBuf::from("/home/test/Documents/important");
    let entry = PathBuf::from("/home/test/Documents/important");

    let ep = EntryPath::new(&root, &entry).unwrap();

    assert_eq!(ep.parts().len(), 0);
}

#[test]
fn test_entry_path_new_fails_with_parent() {
    let root = PathBuf::from("/home/test/Documents/important");
    let entry = PathBuf::from("/home/test/Documents/important/../a/b/c.txt");

    let err = EntryPath::new(&root, &entry).unwrap_err();
    match err {
        Error::NotChild(ref e, ref r) => {
            assert_eq!(e, &PathBuf::from("/home/test/Documents"));
            assert_eq!(r, &root);
        }
        e => panic!("{:?}", e),
    }
}

#[test]
fn test_entry_path_new_success_with_parent() {
    let root = PathBuf::from("/home/test/Documents/important");
    let entry = PathBuf::from("/home/test/Documents/important/a/../b/c.txt");

    let ep = EntryPath::new(&root, &entry).unwrap();
    assert_eq!(ep.parts(), &["b".to_owned(), "c.txt".to_owned()],);
}
