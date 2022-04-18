use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct CargoPackage {
    name: String,
    version: String,
    description: Option<String>,
    authors: Vec<String>,
    keywords: Option<Vec<String>>,
    repository: Option<String>,
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

fn get_manifest_path() -> PathBuf {
    let output = Command::new("cargo")
        .arg("locate-project")
        .output()
        .unwrap();
    serde_json::from_str::<CargoLocation>(&String::from_utf8(output.stdout).unwrap())
        .unwrap()
        .root
        .into()
}

fn read_manifest(manifest: &Path) -> io::Result<Cargo> {
    File::open(manifest)
        .and_then(|mut f| {
            let mut buf = String::with_capacity(1024);
            f.read_to_string(&mut buf).map(|_| buf)
        })
        .map(|buf| toml::from_str(&buf).unwrap())
}

fn escape(s: &str) -> String {
    s.chars().flat_map(char::escape_default).collect()
}

fn generate_pkgbuild(manifest: &Cargo, output: &mut dyn Write) -> io::Result<()> {
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

    write!(
        output,
        r#"
build() {{
    return 0
}}
"#
    )?;

    if let Some(ref repo) = manifest.package.repository {
        write!(
            output,
            r#"
package() {{
    cd $srcdir
    cargo install --root="$pkgdir/usr" --git={}
}}
"#,
            repo
        )?;
    } else {
        write!(
            output,
            r#"
package() {{
    cargo install --root="$pkgdir" {}
}}
"#,
            manifest.package.name
        )?;
    }

    Ok(())
}

fn main() {
    generate_pkgbuild(
        &read_manifest(&get_manifest_path()).unwrap(),
        &mut File::create("PKGBUILD").unwrap(),
    )
    .unwrap();
}
