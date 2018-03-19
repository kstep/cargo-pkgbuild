extern crate rustc_serialize;
extern crate toml;

use std::process::Command;
use std::path::{PathBuf, Path};
use std::fs::File;
use std::io::{self, Read, Write};
use rustc_serialize::json;

#[derive(RustcDecodable, Debug)]
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

#[derive(RustcDecodable, Debug)]
struct Cargo {
    package: CargoPackage,
}

#[derive(RustcDecodable, Debug)]
struct CargoLocation {
    root: String
}

fn get_manifest_path() -> PathBuf {
    let output = Command::new("cargo").arg("locate-project").output().unwrap();
    json::decode::<CargoLocation>(&String::from_utf8(output.stdout).unwrap()).unwrap().root.into()
}

fn read_manifest(manifest: &Path) -> io::Result<Cargo> {
    File::open(manifest)
        .and_then(|mut f| {
            let mut buf = String::with_capacity(1024);
            f.read_to_string(&mut buf).map(|_| buf)
        })
        .map(|buf| {
            toml::decode_str(&buf).unwrap()
        })
}

fn escape(s: &str) -> String {
    s.chars().flat_map(char::escape_default).collect()
}

fn generate_pkgbuild(manifest: &Cargo, output: &mut Write) -> io::Result<()> {
    for author in &manifest.package.authors {
        try!(writeln!(output, "# Maintainer: {}", author));
    }

    try!(writeln!(output, "pkgname={}", manifest.package.name));
    try!(writeln!(output, "pkgver={}", manifest.package.version));
    try!(writeln!(output, "pkgrel=1"));
    try!(writeln!(output, "makedepends=('rust' 'cargo')"));
    try!(writeln!(output, "arch=('i686' 'x86_64' 'armv6h' 'armv7h')"));
    if let Some(ref desc) = manifest.package.description {
        try!(writeln!(output, "pkgdesc=\"{}\"", escape(desc)));
    }
    if let Some(ref url) = manifest.package.homepage {
        try!(writeln!(output, "url=\"{}\"", url));
    }
    if let Some(ref license) = manifest.package.license {
        try!(writeln!(output, "license=('{}')", license));
    }

    try!(write!(output, r#"
build() {{
    return 0
}}
"#));

    if let Some(ref repo) = manifest.package.repository {
        try!(write!(output, r#"
package() {{
    cd $srcdir
    cargo install --root="$pkgdir/usr" --git={}
}}
"#, repo));
    } else {
        try!(write!(output, r#"
package() {{
    cargo install --root="$pkgdir" {}
}}
"#, manifest.package.name));
    }

    Ok(())
}

fn main() {
    generate_pkgbuild(
        &read_manifest(&get_manifest_path()).unwrap(),
        &mut File::create("PKGBUILD").unwrap()
    ).unwrap();
}
