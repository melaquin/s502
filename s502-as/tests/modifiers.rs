use std::fs;

fn cleanup(test_name: &'static str) {
    let _ = fs::remove_file(format!("test_input/{}.65a", test_name));
    let _ = fs::remove_file(format!("test_input/{}_listing.txt", test_name));
    let _ = fs::remove_file(format!("test_input/{}.bin", test_name));
}

#[test]
fn high() {
    let test_name = "high";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            adc #<$1
            adc <$2
            adc <$3,x
            adc <$100
            adc <$200,x
            adc (<$4,x)
            adc (<$5),y
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
        vec![0x69, 0x01, 0x65, 0x02, 0x75, 0x03, 0x65, 0x01, 0x75, 0x02, 0x61, 0x04, 0x71, 0x05]
    );

    cleanup(test_name);
}

#[test]
fn low() {
    let test_name = "low";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            adc #>$1
            adc >$2
            adc >$3,x
            adc >$120
            adc >$230,x
            adc (>$4,x)
            adc (>$5),y
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
        vec![0x69, 0x01, 0x65, 0x02, 0x75, 0x03, 0x65, 0x20, 0x75, 0x30, 0x61, 0x04, 0x71, 0x05]
    );

    cleanup(test_name);
}
