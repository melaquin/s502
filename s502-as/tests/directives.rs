use std::fs;

fn cleanup(test_name: &'static str) {
    let _ = fs::remove_file(format!("test_input/{}.65a", test_name));
    let _ = fs::remove_file(format!("test_input/{}_listing.txt", test_name));
    let _ = fs::remove_file(format!("test_input/{}.bin", test_name));
}

#[test]
fn byte_macro() {
    let test_name = "byte_macro";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            mac1 equ $1
            mac2 equ $2
            mac3 equ <$3040
            mac4 equ >mac3

            adc mac1
            adc #mac2
            adc mac3,x
            adc #mac4
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
        vec![0x65, 0x01, 0x69, 0x02, 0x75, 0x30, 0x69, 0x30]
    );

    cleanup(test_name);
}

#[test]
fn word_macro() {
    let test_name = "word_macro";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            mac1 equ $100
            mac2 equ $203

            adc mac1
            adc #<mac2
            adc mac2,x
            adc >mac2,x
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
        vec![0x6d, 0x00, 0x01, 0x69, 0x02, 0x7d, 0x03, 0x02, 0x75, 0x03]
    );

    cleanup(test_name);
}

#[test]
fn bytes() {
    let test_name = "bytes";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
                dfb $10
                dfw $2030
                dfb \" !\\\"#$%&'()*+,-./0123456789:;<=>?@abcdefghijklmnopqrstuvwxyz[\\\\]^_\"
                dfb \"\\f !\\\"#$%&'()*+,-./0123456789:;<=>?@abcdefghijklmnopqrstuvwxyz[\\\\]^_\"
                dfb \"\\i !\\\"#$%&'()*+,-./0123456789:;<=>?@abcdefghijklmnopqrstuvwxyz[\\\\]^_\"
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

    #[rustfmt::skip]
    assert_eq!(
        fs::read(format!("test_input/{}.bin", test_name)).unwrap(),
        vec![
            // Plain bytes.
            0x10, 0x30, 0x20,
            // Normal string.
            0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa,
            0xab, 0xac, 0xad, 0xae, 0xaf,
            0xb0, 0xb1, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba,
            0xbb, 0xbc, 0xbd, 0xbe, 0xbf,
            0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca,
            0xcb, 0xcc, 0xcd, 0xce, 0xcf,
            0xd0, 0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda,
            0xdb, 0xdc, 0xdd, 0xde, 0xdf,
            // Flashing string.
            0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a,
            0x6b, 0x6c, 0x6d, 0x6e, 0x6f,
            0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a,
            0x7b, 0x7c, 0x7d, 0x7e, 0x7f,
            0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a,
            0x4b, 0x4c, 0x4d, 0x4e, 0x4f,
            0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a,
            0x5b, 0x5c, 0x5d, 0x5e, 0x5f,
            // Inverse string.
            0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a,
            0x2b, 0x2c, 0x2d, 0x2e, 0x2f,
            0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a,
            0x3b, 0x3c, 0x3d, 0x3e, 0x3f,
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a,
            0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
            0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a,
            0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
        ]
    );

    cleanup(test_name);
}

#[test]
fn origin() {
    let test_name = "origin";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
                    org $1030
                one
                    dfw one

                    org $1040
                two
                    dfw two

                    org $1020
                three
                    dfw three
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
            0x20, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x30, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x40, 0x10
        ]
    );

    cleanup(test_name);
}

#[test]
fn include() {
    let test_name = "include";

    assert!(fs::write(
        format!("test_input/{}_0.65a", test_name),
        indoc::formatdoc! {
            "
            dfb $10
            inl \"test_input/{}_1.65a\"
            ", test_name
        },
    )
    .is_ok());

    assert!(fs::write(
        format!("test_input/{}_1.65a", test_name),
        indoc::formatdoc! {
            "
            dfw $2030
            "
        },
    )
    .is_ok());

    let output = test_bin::get_test_bin("s502-as")
        .arg("-b")
        .arg("-l")
        .arg(format!("test_input/{}_0.65a", test_name))
        .output()
        .expect("failed to start s502-as");

    assert!(String::from_utf8_lossy(&output.stderr).is_empty());
    assert!(String::from_utf8_lossy(&output.stdout).is_empty());

    assert_eq!(
        fs::read(format!("test_input/{}_0.bin", test_name)).unwrap(),
        vec![0x10, 0x30, 0x20]
    );

    cleanup("include_0");
    cleanup("include_1");
}
