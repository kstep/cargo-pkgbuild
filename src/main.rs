use std::{fs::File, io::Write, path::PathBuf, process};

use anyhow::{anyhow, Context};
use cargo_metadata::Package;

type Result<T> = anyhow::Result<T>;

const MANIFEST_FILENAME: &str = "Cargo.toml";

fn escape(s: &str) -> String {
    s.chars().flat_map(char::escape_default).collect()
}

fn parse_manifest(manifest_path: &PathBuf) -> Result<Package> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(&manifest_path)
        .exec()?;
    metadata.root_package().cloned().with_context(|| {
        anyhow!(
            "`{}` does not contain a root package",
            manifest_path.display()
        )
    })
}

fn generate_pkgbuild(manifest: &Package, output: &mut dyn Write) -> Result<()> {
    for author in &manifest.authors {
        writeln!(output, "# Maintainer: {}", author)?;
    }

    writeln!(output, "pkgname={}", manifest.name)?;
    writeln!(output, "pkgver={}", manifest.version)?;
    writeln!(output, "pkgrel=1")?;
    writeln!(output, "makedepends=('rust' 'cargo')")?;
    writeln!(output, "arch=('i686' 'x86_64' 'armv6h' 'armv7h')")?;
    if let Some(ref desc) = manifest.description {
        writeln!(output, "pkgdesc=\"{}\"", escape(desc))?;
    }
    if let Some(ref url) = manifest.homepage {
        writeln!(output, "url=\"{}\"", url)?;
    }
    if let Some(ref license) = manifest.license {
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
    fn exec() -> Result<()> {
        let manifest_path = locate_cargo_manifest::locate_manifest()
            .with_context(|| anyhow!("Failed to locate `{MANIFEST_FILENAME}`"))?;
        let manifest = parse_manifest(&manifest_path)
            .with_context(|| anyhow!("Failed to parse `{}`", manifest_path.display()))?;
        let mut pkgbuild = File::create("PKGBUILD").context("Failed to create PKGBUILD file")?;
        generate_pkgbuild(&manifest, &mut pkgbuild).context("Failed to generate PKGBUILD")?;
        Ok(())
    }

    if let Err(e) = exec() {
        eprintln!("Error: {e:?}");
        process::exit(1)
    }
}
