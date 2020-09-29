fn main() {
    match (std::env::var("CARGO_FEATURE_1K"), std::env::var("CARGO_FEATURE_2K")) {
        (Ok(_), Ok(_)) => {
            panic!(r#"

Both buffer size features ('1k' and '2k') are enabled. You may only select one or none.

"#);
        }
        _ => {}
    }
}