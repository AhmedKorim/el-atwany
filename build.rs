use std::env;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/atwany.proto");
    let current_env = env::var("CARGO_CFG_TARGET_ENV")?;
    println!("Current Target env={}", current_env);
    if current_env.to_lowercase() == "musl" {
        // we don't need to build for musl
        return Ok(());
    }
    tonic_build::configure()
        .out_dir("src/pb")
        .format(true)
        .build_server(true)
        .build_client(false)
        .compile(&["proto/atwany.proto"], &["proto"])?;
    Ok(())
}
