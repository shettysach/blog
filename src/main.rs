mod parse;
use parse::*;

fn main() {
    let input_dir = "markdown";
    let output_dir = "_site/";

    std::fs::create_dir_all(output_dir).expect("Failed to create output directory");
    process_index(input_dir, output_dir);
}
