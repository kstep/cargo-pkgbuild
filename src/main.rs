use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

use anyhow::Context;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct CargoPackage {
    name: String,
    version: String,
    description: Option<String>,
    authors: Vec<String>,
    homepage: Option<String>,
    license: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Cargo {
    package: CargoPackage,
}

#[derive(Deserialize, Debug)]
struct CargoLocation {
    root: String,
}

fn get_manifest_path() -> anyhow::Result<PathBuf> {
    let output = Command::new("cargo")
        .arg("locate-project")
        .output()
        .context("Failed to execute `cargo locate-project`")?;
    let manifest =
        String::from_utf8(output.stdout).expect("Invalid utf-8 from `cargo locate-project`");
    let manifest_path = serde_json::from_str::<CargoLocation>(&manifest)
        .context("Failed to deserialize json output from `cargo locate-project`")?
        .root
        .into();
    Ok(manifest_path)
}

fn read_manifest(manifest: &Path) -> anyhow::Result<Cargo> {
    let buf = fs::read_to_string(manifest).context("Failed to open project manifest")?;
    toml::from_str(&buf).context("Failed to deserialize project manifest")
}

fn escape(s: &str) -> String {
    s.chars().flat_map(char::escape_default).collect()
}

fn generate_pkgbuild(manifest: &Cargo, output: &mut dyn Write) -> anyhow::Result<()> {
    for author in &manifest.package.authors {
        writeln!(output, "# Maintainer: {}", author)?;
    }

    writeln!(output, "pkgname={}", manifest.package.name)?;
    writeln!(output, "pkgver={}", manifest.package.version)?;
    writeln!(output, "pkgrel=1")?;
    writeln!(output, "makedepends=('rust' 'cargo')")?;
    writeln!(output, "arch=('i686' 'x86_64' 'armv6h' 'armv7h')")?;
    if let Some(ref desc) = manifest.package.description {
        writeln!(output, "pkgdesc=\"{}\"", escape(desc))?;
    }
    if let Some(ref url) = manifest.package.homepage {
        writeln!(output, "url=\"{}\"", url)?;
    }
    if let Some(ref license) = manifest.package.license {
        writeln!(output, "license=('{}')", license)?;
    }

    writeln!(
        output,
        r#"
# Generated in accordance to https://wiki.archlinux.org/title/Rust_package_guidelines.
# Might require further modification depending on the package involved.
prepare() {{
  cargo fetch --locked --target "$CARCH-unknown-linux-gnu"
}}"#
    )?;

    writeln!(
        output,
        r#"
build() {{
  export RUSTUP_TOOLCHAIN=stable
  export CARGO_TARGET_DIR=target
  cargo build --frozen --release --all-features
}}"#
    )?;

    writeln!(
        output,
        r#"
check() {{
  export RUSTUP_TOOLCHAIN=stable
  cargo test --frozen --all-features
}}"#
    )?;

    writeln!(
        output,
        r#"
package() {{
  install -Dm0755 -t "$pkgdir/usr/bin/" "target/release/$pkgname"
}}"#
    )?;

    Ok(())
}

fn main() {
    fn exec() -> anyhow::Result<()> {
        let manifest_path = get_manifest_path().context("Failed to get manifest path")?;
        let manifest = read_manifest(&manifest_path).context("Failed to read project manifest")?;
        let mut pkgbuild = File::create("PKGBUILD").context("Failed to create PKGBUILD file")?;
        generate_pkgbuild(&manifest, &mut pkgbuild).context("Failed to generate PKGBUILD")?;
        Ok(())
    }

    if let Err(e) = exec() {
        eprintln!("Error: {e:?}");
        process::exit(1)
    }
}
