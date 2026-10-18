// Exporta DirectDrawCreate sem a decoracao stdcall (_DirectDrawCreate@12), como o cm0102.exe importa.
fn main() {
    let def = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("ddraw.def");
    println!("cargo:rustc-cdylib-link-arg=/DEF:{}", def.display());
    println!("cargo:rerun-if-changed=ddraw.def");
}
