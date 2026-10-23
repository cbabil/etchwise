//! Compiles the C password hashers upstream bundles, unchanged: openwall
//! yescrypt (with Raspberry Pi's wrapper) and Ulrich Drepper's sha256crypt.

fn main() {
    let yescrypt = [
        "yescrypt-opt.c",
        "yescrypt-common.c",
        "yescrypt-platform.c",
        "sha256.c",
        "insecure_memzero.c",
        "yescrypt_wrapper.c",
    ];
    cc::Build::new()
        .files(yescrypt.iter().map(|f| format!("vendor/yescrypt/{f}")))
        .include("vendor/yescrypt")
        .warnings(false)
        .compile("yescrypt");
    cc::Build::new()
        .file("vendor/sha256crypt/sha256crypt.c")
        .warnings(false)
        .compile("sha256crypt");
    println!("cargo:rerun-if-changed=vendor");
}
