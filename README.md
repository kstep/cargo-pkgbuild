# cargo-pkgbuild

ArchLinux's PKGBULD generator from Cargo.toml manifest file

At first install [rust and cargo](https://www.rust-lang.org/downloads.html):

```
$ pacman -S rust cargo
```

Then install this package:

```
$ cargo install cargo-pkgbuild
```

Now you can create PKGBUILD from your project Cargo.toml:

```
$ cd my-rust-project
$ cargo pkgbuild
```

Edit the resulting PKGBUILD to your taste and enjoy!
