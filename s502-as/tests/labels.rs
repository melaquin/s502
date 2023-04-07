use std::fs;

fn cleanup(test_name: &'static str) {
    let _ = fs::remove_file(format!("test_input/{}.65a", test_name));
    let _ = fs::remove_file(format!("test_input/{}_listing.txt", test_name));
    let _ = fs::remove_file(format!("test_input/{}.bin", test_name));
}

#[test]
fn parent() {
    let test_name = "parent";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            * full line comment
            one             comment after label
                dfb $10     comment after instruction
            two
                dfw $2030

                adc one
                adc two
                jmp two
                adc #<one
                adc #>two
            "
        },
    )
    .is_ok());

    let output = test_bin::get_test_bin("s502-as")
        .arg("-b")
        .arg("-l")
        .arg(format!("test_input/{}.65a", test_name))
        .output()
        .expect("failed to start s502-as");

    assert!(String::from_utf8_lossy(&output.stderr).is_empty());
    assert!(String::from_utf8_lossy(&output.stdout).is_empty());

    assert_eq!(
        fs::read(format!("test_input/{}.bin", test_name)).unwrap(),
        vec![
            0x10, 0x30, 0x20, 0x6d, 0x00, 0x00, 0x6d, 0x01, 0x00, 0x4c, 0x01, 0x00, 0x69, 0x00,
            0x69, 0x01
        ]
    );

    cleanup(test_name);
}

#[test]
fn child() {
    let test_name = "child";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            one
                dfb $10
            .two
                dfw $2030
            one.three

                jmp one
                jmp .two
                adc #>.three
            "
        },
    )
    .is_ok());

    let output = test_bin::get_test_bin("s502-as")
        .arg("-b")
        .arg("-l")
        .arg(format!("test_input/{}.65a", test_name))
        .output()
        .expect("failed to start s502-as");

    assert!(String::from_utf8_lossy(&output.stderr).is_empty());
    assert!(String::from_utf8_lossy(&output.stdout).is_empty());

    assert_eq!(
        fs::read(format!("test_input/{}.bin", test_name)).unwrap(),
        vec![0x10, 0x30, 0x20, 0x4c, 0x00, 0x00, 0x4c, 0x01, 0x00, 0x69, 0x03]
    );

    cleanup(test_name);
}
