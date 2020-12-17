use anyhow::Result;
use argh::FromArgs;
use std::{fs, path::PathBuf, process::Command};

#[derive(FromArgs)]
/// A Simple Task Manager.
struct Tasker {
    /// build for musl target
    #[argh(switch)]
    build_musl: bool,
    /// build it in a release mode
    #[argh(switch)]
    release: bool,
    /// build the docker image of the current build
    #[argh(switch)]
    dockerize: bool,
}

fn main() -> Result<()> {
    let args: Tasker = argh::from_env();
    if args.build_musl {
        build_musl(&args)?;
    }
    if args.dockerize {
        dockerize(&args)?;
    }
    Ok(())
}

fn build_musl(args: &Tasker) -> Result<()> {
    fs::create_dir_all("build")?;
    let cmd_args = [
        "build",
        "-p",
        "atwany",
        if args.release { "--release" } else { "" },
        "--target",
        "x86_64-unknown-linux-musl",
    ];
    let mut cross = Command::new("cross")
        // don't use any rustc wrapper
        .env("RUSTC_WRAPPER", "")
        .args(cmd_args.iter().filter(|c| !c.is_empty()))
        .spawn()?;
    let status = cross.wait()?;
    assert!(status.success());
    let mut p = PathBuf::new();
    p.push("target/x86_64-unknown-linux-musl");
    if args.release {
        p.push("release");
    } else {
        p.push("debug");
    }
    p.push("atwany");
    fs::copy(p, "build/atwany")?;
    Ok(())
}

fn dockerize(args: &Tasker) -> Result<()> {
    let cmd_args = [
        "build",
        "-t",
        if args.release {
            "atwany/atwany-release:latest"
        } else {
            "atwany/atwany-debug:latest"
        },
        "-f",
        "ATWANY.Dockerfile",
        ".",
    ];
    let mut docker = Command::new("docker").args(&cmd_args).spawn()?;
    let status = docker.wait()?;
    assert!(status.success());
    Ok(())
}
