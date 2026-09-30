//! Registers every card port in `src/cards/impls/*.rs` (each defines
//! `pub static IMPL: CardImpl`), so ports never edit a shared file.
use std::fs;
use std::path::Path;

fn main() {
    let dir = Path::new("src/cards/impls");
    println!("cargo:rerun-if-changed=src/cards/impls");
    let mut names: Vec<String> = fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .filter(|n| n.ends_with(".rs") && n != "mod.rs")
                .map(|n| n.trim_end_matches(".rs").to_string())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut out = String::new();
    for n in &names {
        out.push_str(&format!("#[path = \"{}/src/cards/impls/{}.rs\"]\npub mod {};\n", root, n, n));
    }
    out.push_str("pub static IMPLS: &[&crate::cards::CardImpl] = &[\n");
    for n in &names {
        out.push_str(&format!("    &{}::IMPL,\n", n));
    }
    out.push_str("];\n");
    let dest = Path::new(&std::env::var("OUT_DIR").unwrap()).join("card_impls.rs");
    fs::write(dest, out).unwrap();
}
