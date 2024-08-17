pub fn print_buf(buf: &[u8], prefix: &str) {
    println!(
        "  (DEBUG) {prefix}: {:?}",
        buf.iter().map(|b| *b as char).collect::<String>()
    );
}
