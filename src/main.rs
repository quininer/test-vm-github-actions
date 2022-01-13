mod util;

use std::{ thread, env, fs };
use std::io::Write;
use std::path::Path;
use std::process::{ Command, Stdio };
use anyhow::Context;
use tracing::{ debug, info };
use crate::util::ExitStatusExt;


const KERNEL_TARBALL_URL: &str = "https://cdn.kernel.org/pub/linux/kernel/v5.x/linux-5.15.14.tar.xz";

fn build_kernel(pwd: &Path) -> anyhow::Result<()> {
    let kernel_dir = pwd.join("kernel-build");

    let _ = fs::create_dir_all(&kernel_dir);

    // download tarball
    let mut cmd = Command::new("curl");
    cmd
        .current_dir(&kernel_dir)
        .arg("-L")
        .arg("-O")
        .arg("-C")
        .arg("-")
        .arg(KERNEL_TARBALL_URL);
    info!(?cmd, "download kernel tarball");
    cmd.status()?.exit_ok2()?;

    // extract kernel
    let kernel_tarball = kernel_dir.read_dir()?
        .take(128)
        .filter_map(Result::ok)
        .filter(|file| file.file_type().map_or(false, |ty| ty.is_file()))
        .map(|file| file.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .filter(|name| name.ends_with(".tar.xz"))
                .is_some()
        })
        .next()
        .context("not found kernel tarball")?;
    let mut cmd = Command::new("tar");
    cmd.arg("-xf")
        .current_dir(&kernel_dir)
        .arg(&kernel_tarball);
    info!(?cmd, "extract kernel");
    cmd.status()?.exit_ok2()?;

    let list = kernel_dir.read_dir()?.take(128).collect::<Vec<_>>();
    info!(?list, "list dir");

    // make kernel
    let kernel_dir_linux = kernel_dir.read_dir()?
        .take(128)
        .filter_map(Result::ok)
        .filter(|file| file.file_type().map_or(false, |ty| ty.is_dir()))
        .map(|file| file.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .filter(|name| name.starts_with("linux-"))
                .is_some()
        })
        .next()
        .context("not found kernel tarball")?;
    fs::copy(pwd.join("microvm-kernel-x86_64-5.10.config"), kernel_dir_linux.join(".config"))?;
    let mut child = Command::new("make")
        .current_dir(&kernel_dir_linux)
        .stdin(Stdio::piped())
        .arg("vmlinux")
        .spawn()?;
    let mut make_stdin = child.stdin.take().context("make stdin is none")?;
    thread::spawn(move || {
        while make_stdin.write_all(b"\n").is_ok() {
            thread::yield_now();
        }
    });
    child.wait()?.exit_ok2()?;

    // mv vmlinux
    fs::copy(kernel_dir_linux.join("vmlinux"), kernel_dir.join("vmlinux"))?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let pwd = env::current_dir()?;

    build_kernel(&pwd)?;

    Ok(())
}
