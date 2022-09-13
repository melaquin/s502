#[test]
fn test_labels_file() {
    let output = test_bin::get_test_bin("s502-as")
        .arg("test_inputs/labels.65a")
        .output()
        .expect("failed to start s502-as");

    assert!(String::from_utf8_lossy(&output.stderr).is_empty());
}

#[test]
fn test_labels_with_comments() {
    let output = test_bin::get_test_bin("s502-as")
        .arg("test_inputs/labels_with_comments.65a")
        .output()
        .expect("failed to start s502-as");

    assert!(String::from_utf8_lossy(&output.stderr).is_empty());
}
