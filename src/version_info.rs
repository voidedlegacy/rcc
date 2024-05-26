const VERSION_STR: &str = "0.1.0";

pub fn show_version() {
    println!("rcc: (v{})", VERSION_STR);
}

pub fn show_usage() {
    println!("Usage: rcc <input_file>");
}
