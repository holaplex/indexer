use std::{
    env,
    fmt::Display,
    process::{Command, Output},
    str,
};

fn run(c: &mut Command, name: impl Display) -> Output {
    let output = c.output().unwrap();

    if !output.status.success() {
        str::from_utf8(&*output.stdout)
            .map(|s| print!("{}", s))
            .ok();
        str::from_utf8(&*output.stderr)
            .map(|s| eprint!("{}", s))
            .ok();
        println!(
            "{} exited with code {}",
            name,
            output
                .status
                .code()
                .map_or_else(|| "???".into(), |s| s.to_string())
        );
    }

    output
}

fn main() {
    for var in &["HOST", "TARGET", "PROFILE"] {
        println!(
            "cargo:rustc-env=META_BUILD_{}={}",
            var,
            env::var(var).unwrap()
        );
    }

    println!(
        "cargo:rustc-env=META_BUILD_PLATFORM=ptr{},{},{}",
        env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap(),
        env::var("CARGO_CFG_TARGET_ENDIAN").unwrap(),
        env::var("CARGO_CFG_TARGET_FEATURE").unwrap(),
    );

    let toplevel = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output();

    match toplevel {
        Ok(toplevel) if toplevel.status.success() => {
            let toplevel = str::from_utf8(&toplevel.stdout).unwrap().trim();

            let rev = run(
                Command::new("git")
                    .args(&["rev-parse", "--short", "HEAD"])
                    .current_dir(toplevel),
                "git rev-parse",
            );

            let status = run(
                Command::new("git")
                    .args(&["status", "--porcelain"])
                    .current_dir(toplevel),
                "git status",
            );

            let ls_files = run(
                Command::new("git")
                    .args(&["ls-files", "--full-name"])
                    .current_dir(toplevel),
                "git ls-files",
            );

            println!("cargo:rerun-if-changed={}/.git/index", toplevel);

            for file in str::from_utf8(&ls_files.stdout)
                .unwrap()
                .split('\n')
                .map(|f| f.trim())
            {
                println!("cargo:rerun-if-changed={}/{}", toplevel, file);
            }

            let status = str::from_utf8(&status.stdout).unwrap();
            for file in status
                .split('\n')
                .map(|f| f.trim())
                .filter(|f| f.len() > 2)
                .map(|f| f[2..].trim())
            {
                println!("cargo:rerun-if-changed={}/{}", toplevel, file);
            }

            println!(
                "cargo:rustc-env=META_GIT_HEAD={}{}",
                str::from_utf8(&rev.stdout).unwrap().trim(),
                if status.trim().is_empty() {
                    ""
                } else {
                    "-DIRTY"
                }
            );

            let branch = run(
                Command::new("git").args(&[
                    "branch",
                    "--show-current",
                    "--format=%(refname:short)",
                ]),
                "git branch",
            );

            let remote = run(
                Command::new("git").arg("config").arg(format!(
                    "branch.{}.remote",
                    str::from_utf8(&branch.stdout).unwrap().trim()
                )),
                "git config",
            );

            let remote_url = run(
                Command::new("git")
                    .args(&["remote", "get-url"])
                    .arg(str::from_utf8(&remote.stdout).unwrap().trim()),
                "git remote",
            );

            println!(
                "cargo:rustc-env=META_GIT_REMOTE={}",
                str::from_utf8(&remote_url.stdout).unwrap().trim()
            );

            println!("cargo:rerun-if-changed={}/.git/config", toplevel);
        },
        _ => (),
    }

    let ver = run(
        Command::new(env::var("RUSTC").unwrap()).arg("--version"),
        "rustc --version",
    );

    println!(
        "cargo:rustc-env=META_RUSTC_VERSION={}",
        str::from_utf8(&ver.stdout).unwrap().trim()
    );
}
