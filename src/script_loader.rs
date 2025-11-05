// Script loader - embeds scripts at compile time and loads them into filesystem

use crate::fs;

/// Embedded script files
pub const SCRIPTS: &[(&str, &[u8])] = &[
    ("fibonacci.rscript", include_bytes!("rustrial_script/examples/fibonacci.rscript")),
    ("factorial.rscript", include_bytes!("rustrial_script/examples/factorial.rscript")),
    ("collatz.rscript", include_bytes!("rustrial_script/examples/collatz.rscript")),
    ("gcd.rscript", include_bytes!("rustrial_script/examples/gcd.rscript")),
    ("prime_checker.rscript", include_bytes!("rustrial_script/examples/prime_checker.rscript")),
    ("sum_of_squares.rscript", include_bytes!("rustrial_script/examples/sum_of_squares.rscript")),
];

/// Load all embedded scripts into the filesystem
pub fn load_scripts() -> Result<(), fs::VfsError> {
    crate::println!("[LOADER] Loading embedded scripts...");
    fs::mount_scripts(SCRIPTS)?;
    crate::println!("[LOADER] Loaded {} scripts", SCRIPTS.len());
    Ok(())
}

/// Get list of available script names
pub fn list_scripts() -> alloc::vec::Vec<alloc::string::String> {
    use alloc::string::ToString;
    SCRIPTS.iter().map(|(name, _)| (*name).to_string()).collect()
}

/// Get script content by name
pub fn get_script_content(name: &str) -> Option<&'static [u8]> {
    SCRIPTS.iter()
        .find(|(n, _)| *n == name)
        .map(|(_, content)| *content)
}
