fn main() {
    // Capture build timestamp as a compile-time env var
    let output = std::process::Command::new("date")
        .args(["-u", "+%Y-%m-%d %H:%M UTC"])
        .output()
        .expect("failed to run date command");
    let timestamp = String::from_utf8(output.stdout)
        .expect("invalid UTF-8 from date")
        .trim()
        .to_string();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", timestamp);
}
