use std::{
    env,
    fmt::Display,
    fs::File,
    io::prelude::*,
    path::Path,
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

fn check_schema(expected: String, path: impl AsRef<Path>) {
    const PATH: &str = "src/config/config-schema.graphql";

    let mut s = String::new();

    File::open(PATH)
        .and_then(|mut f| f.read_to_string(&mut s))
        .map_err(|e| format!("{:?}", e))
        .and_then(|_| {
            if s == expected {
                Ok(())
            } else {
                Err("Mismatch in config SDL definitions".into())
            }
        })
        .map_err(|s| {
            format!(
                "Failed checking schema {}: {} - consult {} for a correct definition",
                PATH,
                s,
                path.as_ref().display()
            )
        })
        .unwrap();
}

// TODO: at the moment graphql_client doesn't support reading files from $OUT_DIR
fn write_schema() {
    let dir = env::var("OUT_DIR").unwrap();
    let path = Path::new(&dir).join("config-schema.graphql");
    let mut f = File::create(&path).unwrap();

    let sdl = geyser_config::schema::build(geyser_config::schema::SchemaData {}).sdl();

    f.write_all(sdl.as_bytes()).unwrap();

    check_schema(sdl, &path);
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
        .output()
        .unwrap();

    if toplevel.status.success() {
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

        for file in str::from_utf8(&status.stdout)
            .unwrap()
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
            if status.stdout.is_empty() {
                ""
            } else {
                "-DIRTY"
            }
        );

        let branch = run(
            Command::new("git").args(&["branch", "--show-current", "--format=%(refname:short)"]),
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
    }

    let ver = run(
        Command::new(env::var("RUSTC").unwrap()).arg("--version"),
        "rustc --version",
    );

    println!(
        "cargo:rustc-env=META_RUSTC_VERSION={}",
        str::from_utf8(&ver.stdout).unwrap().trim()
    );

    write_schema();
}
