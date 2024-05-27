extern crate rcc;
use rcc::version_info;
use rcc::lexer;

use std::fs::OpenOptions;
use std::io::Read;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        version_info::show_version();
        version_info::show_usage();
    } else {
        let input_file_name = &args[1]; // Borrow the string to avoid unnecessary cloning

        // Open the file and read its content
        let mut file = OpenOptions::new()
            .read(true)
            .open(input_file_name)
            .expect("Failed to open the file");

        let mut s = String::new();
        file.read_to_string(&mut s).expect("Failed to read the file");
        let mut lexer = lexer::Lexer::new(input_file_name.clone(), &s); 
        // test 
        let mut tok: Option<lexer::Token>;
        loop {
            match lexer.get() {
                Some(t) => {
                    println!("token: {}{}", if t.space { " "} else {""}, t.val);
                }
                None => break,
            }
        }
    }
}
