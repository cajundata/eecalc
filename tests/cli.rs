use std::process::Command;

fn run(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_eecalc"))
        .args(args)
        .output()
        .expect("failed to spawn eecalc");
    let stdout = String::from_utf8(output.stdout).expect("stdout utf-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr utf-8");
    let code = output.status.code().unwrap_or(-1);
    (code, stdout, stderr)
}

fn assert_success(args: &[&str], stdout: &str) {
    let (code, out, err) = run(args);
    assert_eq!(err, "", "args={args:?}");
    assert_eq!(code, 0, "args={args:?} stdout={out:?}");
    assert_eq!(out, format!("{stdout}\n"), "args={args:?}");
}

fn assert_error(args: &[&str], code: i32, stderr: &str) {
    let (got_code, out, err) = run(args);
    assert_eq!(out, "", "args={args:?}");
    assert_eq!(got_code, code, "args={args:?} stderr={err:?}");
    assert_eq!(err, format!("error: {stderr}\n"), "args={args:?}");
}

#[test]
fn readme_success_examples() {
    assert_success(
        &["ohm", "--voltage", "12", "--resistance", "470"],
        "I = 25.53 mA",
    );
    assert_success(
        &["ohm", "--current", "20m", "--resistance", "330"],
        "V = 6.6 V",
    );
    assert_success(
        &["power", "--voltage", "5", "--resistance", "100"],
        "P = 250 mW",
    );
    assert_success(&["series", "100", "220", "330"], "R = 650 Ω");
    assert_success(&["parallel", "1k", "1k", "2.2k"], "R = 407.4 Ω");
    assert_success(
        &["divider", "--vin", "5", "--r1", "10k", "--r2", "2.2k"],
        "Vout = 901.6 mV",
    );
    assert_success(
        &["rc", "--resistance", "10k", "--capacitance", "1u"],
        "tau = 10 ms\nfc  = 15.92 Hz",
    );
}

#[test]
fn readme_error_examples() {
    assert_error(
        &["ohm", "--voltage", "12"],
        2,
        "ohm requires exactly two of --voltage, --current, --resistance",
    );
    assert_error(
        &["ohm", "--voltage", "12", "--resistance", "0"],
        1,
        "resistance must be greater than zero",
    );
    assert_error(
        &["rc", "--resistance", "10x", "--capacitance", "1u"],
        2,
        "invalid value '10x': unknown SI prefix 'x'",
    );
}

#[test]
fn one_success_and_failure_per_command() {
    assert_success(&["ohm", "-v", "12", "-i", "2"], "R = 6 Ω");
    assert_error(
        &["ohm"],
        2,
        "ohm requires exactly two of --voltage, --current, --resistance",
    );

    assert_success(&["power", "-i", "20m", "-r", "100"], "P = 40 mW");
    assert_error(
        &["power", "--voltage", "5"],
        2,
        "power requires exactly two of --voltage, --current, --resistance",
    );

    assert_success(&["series", "10", "20"], "R = 30 Ω");
    assert_error(&["series", "10"], 2, "series requires at least two values");

    assert_success(&["parallel", "100", "100"], "R = 50 Ω");
    assert_error(&["parallel"], 2, "parallel requires at least two values");

    assert_success(
        &["divider", "--vin", "10", "--r1", "1k", "--r2", "1k"],
        "Vout = 5 V",
    );
    assert_error(
        &["divider", "--vin", "5"],
        2,
        "divider requires --vin, --r1, and --r2",
    );

    assert_success(
        &["rc", "-r", "1k", "-c", "1u"],
        "tau = 1 ms\nfc  = 159.2 Hz",
    );
    assert_error(
        &["rc", "--capacitance", "1u"],
        2,
        "rc requires --resistance and --capacitance",
    );
}

#[test]
fn help_documents_commands_and_prefixes() {
    let (code, out, err) = run(&["help"]);
    assert_eq!(code, 0);
    assert_eq!(err, "");
    assert!(out.contains("ohm"));
    assert!(out.contains("power"));
    assert!(out.contains("series"));
    assert!(out.contains("parallel"));
    assert!(out.contains("divider"));
    assert!(out.contains("rc"));
    assert!(out.contains("p n u m k M"));
    assert!(out.contains("u means micro"));
}
