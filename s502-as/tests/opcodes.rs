use std::fs;

fn cleanup(test_name: &'static str) {
    let _ = fs::remove_file(format!("test_input/{}.65a", test_name));
    let _ = fs::remove_file(format!("test_input/{}_listing.txt", test_name));
    let _ = fs::remove_file(format!("test_input/{}.bin", test_name));
}

#[test]
fn adc() {
    let test_name = "adc";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            adc #$1
            adc $2
            adc $3,x
            adc $100
            adc $200,x
            adc $300,y
            adc ($4,x)
            adc ($5),y
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
            0x69, 0x01, 0x65, 0x02, 0x75, 0x03, 0x6d, 0x00, 0x01, 0x7d, 0x00, 0x02, 0x79, 0x00,
            0x03, 0x61, 0x04, 0x71, 0x05
        ]
    );

    cleanup(test_name);
}

#[test]
fn and() {
    let test_name = "and";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            and #$1
            and $2
            and $3,x
            and $100
            and $200,x
            and $300,y
            and ($4,x)
            and ($5),y
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
            0x29, 0x01, 0x25, 0x02, 0x35, 0x03, 0x2d, 0x00, 0x01, 0x3d, 0x00, 0x02, 0x39, 0x00,
            0x03, 0x21, 0x04, 0x31, 0x05
        ]
    );

    cleanup(test_name);
}

#[test]
fn asl() {
    let test_name = "asl";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            asl a
            asl $2
            asl $3,x
            asl $100
            asl $200,x
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
        vec![0x0a, 0x06, 0x02, 0x16, 0x03, 0x0e, 0x00, 0x01, 0x1e, 0x00, 0x02]
    );

    cleanup(test_name);
}

#[test]
fn bcc() {
    let test_name = "bcc";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            bcc $10
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
        vec![0x90, 0x10]
    );

    cleanup(test_name);
}

#[test]
fn bcs() {
    let test_name = "bcs";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            bcs $10
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
        vec![0xb0, 0x10]
    );

    cleanup(test_name);
}

#[test]
fn beq() {
    let test_name = "beq";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            beq $10
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
        vec![0xf0, 0x10]
    );

    cleanup(test_name);
}

#[test]
fn bit() {
    let test_name = "bit";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            bit $10
            bit $100
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
        vec![0x24, 0x10, 0x2c, 0x00, 0x01]
    );

    cleanup(test_name);
}

#[test]
fn bmi() {
    let test_name = "bmi";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            bmi $10
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
        vec![0x30, 0x10]
    );

    cleanup(test_name);
}

#[test]
fn bne() {
    let test_name = "bne";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            bne $10
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
        vec![0xd0, 0x10]
    );

    cleanup(test_name);
}

#[test]
fn bpl() {
    let test_name = "bpl";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            bpl $10
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
        vec![0x10, 0x10]
    );

    cleanup(test_name);
}

#[test]
fn brk() {
    let test_name = "brk";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            brk
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
        vec![0x00]
    );

    cleanup(test_name);
}

#[test]
fn bvc() {
    let test_name = "bvc";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            bvc $10
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
        vec![0x50, 0x10]
    );

    cleanup(test_name);
}

#[test]
fn bvs() {
    let test_name = "bvs";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            bvs $10
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
        vec![0x70, 0x10]
    );

    cleanup(test_name);
}

#[test]
fn clc() {
    let test_name = "clc";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            clc
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
        vec![0x18]
    );

    cleanup(test_name);
}

#[test]
fn cld() {
    let test_name = "cld";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            cld
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
        vec![0xd8]
    );

    cleanup(test_name);
}

#[test]
fn cli() {
    let test_name = "cli";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            cli
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
        vec![0x58]
    );

    cleanup(test_name);
}

#[test]
fn clv() {
    let test_name = "clv";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            clv
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
        vec![0xb8]
    );

    cleanup(test_name);
}

#[test]
fn cmp() {
    let test_name = "cmp";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            cmp #$1
            cmp $2
            cmp $3,x
            cmp $100
            cmp $200,x
            cmp $300,y
            cmp ($4,x)
            cmp ($5),y
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
            0xc9, 0x01, 0xc5, 0x02, 0xd5, 0x03, 0xcd, 0x00, 0x01, 0xdd, 0x00, 0x02, 0xd9, 0x00,
            0x03, 0xc1, 0x04, 0xd1, 0x05
        ]
    );

    cleanup(test_name);
}

#[test]
fn cpx() {
    let test_name = "cpx";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            cpx #$1
            cpx $2
            cpx $100
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
        vec![0xe0, 0x01, 0xe4, 0x02, 0xec, 0x00, 0x01]
    );

    cleanup(test_name);
}

#[test]
fn cpy() {
    let test_name = "cpy";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            cpy #$1
            cpy $2
            cpy $100
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
        vec![0xc0, 0x01, 0xc4, 0x02, 0xcc, 0x00, 0x01]
    );

    cleanup(test_name);
}
#[test]
fn dec() {
    let test_name = "dec";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            dec $2
            dec $3,x
            dec $100
            dec $200,x
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
        vec![0xc6, 0x02, 0xd6, 0x03, 0xce, 0x00, 0x01, 0xde, 0x00, 0x02]
    );

    cleanup(test_name);
}

#[test]
fn dex() {
    let test_name = "dex";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            dex
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
        vec![0xca]
    );

    cleanup(test_name);
}

#[test]
fn dey() {
    let test_name = "dey";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            dey
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
        vec![0x88]
    );

    cleanup(test_name);
}

#[test]
fn eor() {
    let test_name = "eor";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            eor #$1
            eor $2
            eor $3,x
            eor $100
            eor $200,x
            eor $300,y
            eor ($4,x)
            eor ($5),y
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
            0x49, 0x01, 0x45, 0x02, 0x55, 0x03, 0x4d, 0x00, 0x01, 0x5d, 0x00, 0x02, 0x59, 0x00,
            0x03, 0x41, 0x04, 0x51, 0x05
        ]
    );

    cleanup(test_name);
}

#[test]
fn inc() {
    let test_name = "inc";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            inc $2
            inc $3,x
            inc $100
            inc $200,x
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
        vec![0xe6, 0x02, 0xf6, 0x03, 0xee, 0x00, 0x01, 0xfe, 0x00, 0x02]
    );

    cleanup(test_name);
}

#[test]
fn inx() {
    let test_name = "inx";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            inx
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
        vec![0xe8]
    );

    cleanup(test_name);
}

#[test]
fn iny() {
    let test_name = "iny";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            iny
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
        vec![0xc8]
    );

    cleanup(test_name);
}

#[test]
fn jmp() {
    let test_name = "jmp";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            jmp $100
            jmp ($200)
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
        vec![0x4c, 0x00, 0x01, 0x6c, 0x00, 0x02]
    );

    cleanup(test_name);
}

#[test]
fn jsr() {
    let test_name = "jsr";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            jsr $100
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
        vec![0x20, 0x00, 0x01]
    );

    cleanup(test_name);
}

#[test]
fn lda() {
    let test_name = "lda";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            lda #$1
            lda $2
            lda $3,x
            lda $100
            lda $200,x
            lda $300,y
            lda ($4,x)
            lda ($5),y
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
            0xa9, 0x01, 0xa5, 0x02, 0xb5, 0x03, 0xad, 0x00, 0x01, 0xbd, 0x00, 0x02, 0xb9, 0x00,
            0x03, 0xa1, 0x04, 0xb1, 0x05
        ]
    );

    cleanup(test_name);
}

#[test]
fn ldx() {
    let test_name = "ldx";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            ldx #$1
            ldx $2
            ldx $3,y
            ldx $100
            ldx $200,y
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
        vec![0xa2, 0x01, 0xa6, 0x02, 0xb6, 0x03, 0xae, 0x00, 0x01, 0xbe, 0x00, 0x02]
    );

    cleanup(test_name);
}

#[test]
fn ldy() {
    let test_name = "ldy";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            ldy #$1
            ldy $2
            ldy $3,x
            ldy $100
            ldy $200,x
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
        vec![0xa0, 0x01, 0xa4, 0x02, 0xb4, 0x03, 0xac, 0x00, 0x01, 0xbc, 0x00, 0x02]
    );

    cleanup(test_name);
}

#[test]
fn lsr() {
    let test_name = "lsr";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            lsr a
            lsr $2
            lsr $3,x
            lsr $100
            lsr $200,x
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
        vec![0x4a, 0x46, 0x02, 0x56, 0x03, 0x4e, 0x00, 0x01, 0x5e, 0x00, 0x02]
    );

    cleanup(test_name);
}

#[test]
fn nop() {
    let test_name = "nop";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            nop
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
        vec![0xea]
    );

    cleanup(test_name);
}

#[test]
fn ora() {
    let test_name = "ora";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            ora #$1
            ora $2
            ora $3,x
            ora $100
            ora $200,x
            ora $300,y
            ora ($4,x)
            ora ($5),y
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
            0x09, 0x01, 0x05, 0x02, 0x15, 0x03, 0x0d, 0x00, 0x01, 0x1d, 0x00, 0x02, 0x19, 0x00,
            0x03, 0x01, 0x04, 0x11, 0x05
        ]
    );

    cleanup(test_name);
}
#[test]

fn pha() {
    let test_name = "pha";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            pha
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
        vec![0x48]
    );

    cleanup(test_name);
}

#[test]
fn php() {
    let test_name = "php";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            php
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
        vec![0x08]
    );

    cleanup(test_name);
}

#[test]
fn pla() {
    let test_name = "pla";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            pla
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
        vec![0x68]
    );

    cleanup(test_name);
}

#[test]
fn plp() {
    let test_name = "plp";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            plp
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
        vec![0x28]
    );

    cleanup(test_name);
}

#[test]
fn rol() {
    let test_name = "rol";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            rol a
            rol $2
            rol $3,x
            rol $100
            rol $200,x
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
        vec![0x2a, 0x26, 0x02, 0x36, 0x03, 0x2e, 0x00, 0x01, 0x3e, 0x00, 0x02]
    );

    cleanup(test_name);
}

#[test]
fn ror() {
    let test_name = "ror";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            ror a
            ror $2
            ror $3,x
            ror $100
            ror $200,x
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
        vec![0x6a, 0x66, 0x02, 0x76, 0x03, 0x6e, 0x00, 0x01, 0x7e, 0x00, 0x02]
    );

    cleanup(test_name);
}

#[test]
fn rti() {
    let test_name = "rti";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            rti
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
        vec![0x40]
    );

    cleanup(test_name);
}

#[test]
fn rts() {
    let test_name = "rts";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            rts
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
        vec![0x60]
    );

    cleanup(test_name);
}

#[test]
fn sbc() {
    let test_name = "sbc";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            sbc #$1
            sbc $2
            sbc $3,x
            sbc $100
            sbc $200,x
            sbc $300,y
            sbc ($4,x)
            sbc ($5),y
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
            0xe9, 0x01, 0xe5, 0x02, 0xf5, 0x03, 0xed, 0x00, 0x01, 0xfd, 0x00, 0x02, 0xf9, 0x00,
            0x03, 0xe1, 0x04, 0xf1, 0x05
        ]
    );

    cleanup(test_name);
}

#[test]
fn sec() {
    let test_name = "sec";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            sec
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
        vec![0x38]
    );

    cleanup(test_name);
}

#[test]
fn sed() {
    let test_name = "sed";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            sed
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
        vec![0xf8]
    );

    cleanup(test_name);
}

#[test]
fn sei() {
    let test_name = "sei";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            sei
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
        vec![0x78]
    );

    cleanup(test_name);
}

#[test]
fn sta() {
    let test_name = "sta";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            sta $2
            sta $3,x
            sta $100
            sta $200,x
            sta $300,y
            sta ($4,x)
            sta ($5),y
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
            0x85, 0x02, 0x95, 0x03, 0x8d, 0x00, 0x01, 0x9d, 0x00, 0x02, 0x99, 0x00, 0x03, 0x81,
            0x04, 0x91, 0x05
        ]
    );

    cleanup(test_name);
}

#[test]
fn stx() {
    let test_name = "stx";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            stx $2
            stx $3,y
            stx $100
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
        vec![0x86, 0x02, 0x96, 0x03, 0x8e, 0x00, 0x01]
    );

    cleanup(test_name);
}

#[test]
fn sty() {
    let test_name = "sty";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            sty $2
            sty $3,x
            sty $100
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
        vec![0x84, 0x02, 0x94, 0x03, 0x8c, 0x00, 0x01]
    );

    cleanup(test_name);
}

#[test]
fn tax() {
    let test_name = "tax";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            tax
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
        vec![0xaa]
    );

    cleanup(test_name);
}

#[test]
fn tay() {
    let test_name = "tay";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            tay
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
        vec![0xa8]
    );

    cleanup(test_name);
}

#[test]
fn tsx() {
    let test_name = "tsx";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            tsx
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
        vec![0xba]
    );

    cleanup(test_name);
}

#[test]
fn txa() {
    let test_name = "txa";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            txa
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
        vec![0x8a]
    );

    cleanup(test_name);
}

#[test]
fn txs() {
    let test_name = "txs";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            txs
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
        vec![0x9a]
    );

    cleanup(test_name);
}

#[test]
fn tya() {
    let test_name = "tya";

    assert!(fs::write(
        format!("test_input/{}.65a", test_name),
        indoc::formatdoc! {
            "
            tya
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
        vec![0x98]
    );

    cleanup(test_name);
}
