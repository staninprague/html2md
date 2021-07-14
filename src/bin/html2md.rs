extern crate html2md;

use std::io::{self, Read};
use std::fs::File;
use std::io::prelude::*;

use std::env;
use std::path::Path;

fn main() -> std::io::Result<()>  {

    let args: Vec<String> = env::args().collect();

    let path = Path::new(&args[1]);

    println!("Will convert file at path: {:?}", &path);

    let mut file_in = File::open(path)?;

    let mut buffer = String::new();
    let _count_of_bytes  = file_in.read_to_string(&mut buffer)?;

    let mut file = File::create(path.with_extension("md"))?;
    file.write_all(html2md::parse_html(&buffer).as_bytes())?;

    println!("Done!" );

    Ok(())
}