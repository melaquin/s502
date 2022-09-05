#[test]
fn labels_file() {
    let output = test_bin::get_test_bin("s502-as")
        .arg("testfiles/labels.65a")
        .output()
        .expect("failed to start s502-as");

    assert!(String::from_utf8_lossy(&output.stderr).is_empty());
}
