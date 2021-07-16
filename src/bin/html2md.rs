extern crate html2md;

use std::fs::File;
use std::io::prelude::*;
use std::io::Read;

use std::env;
use std::path::Path;

use html2md::InputFilePath;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let path = Path::new(&args[1]);

    let path_components: Vec<Option<&str>> = path
        .components()
        .map(|comp| comp.as_os_str().to_str().to_owned())
        .collect();

    println!("Will convert file at path: {:?}", &path);
    let mut path_iter = path_components.iter();
    if let Some(file_name_without_extension) = &path
        .file_stem()
        .expect("Input file name should be present")
        .to_str()
    {
        // Skip over the file name
        &path_iter.next_back();

        if let Some(path_prefix) = &path_iter
            .next_back()
            .expect("parent directory should be present for the input file.")
        {
            println!(
                "Path last component without extension: {:?}",
                &file_name_without_extension
            );

            println!("Path pre-last component: {:?}", &path_prefix);

            let mut file_in = File::open(path)?;

            let mut buffer = String::new();
            let _count_of_bytes = file_in.read_to_string(&mut buffer)?;

            let mut file = File::create(path.with_extension("md"))?;
            file.write_all(
                html2md::parse_html(
                    &buffer,
                    &InputFilePath {
                        parent_dir: path_prefix.to_string(),
                        filename_with_no_extension: file_name_without_extension.to_string(),
                    },
                )
                .as_bytes(),
            )?;

            println!("Done!");
        }
    }

    Ok(())
}
