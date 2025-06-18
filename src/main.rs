use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

// Command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // JPEG image files to process
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

fn is_jpeg(path: &PathBuf) -> Result<bool> {
    let mut file = File::open(path)?;
    let mut buffer = [0; 2];
    file.read_exact(&mut buffer)?;
    
    // JPEG files start with FF D8
    Ok(buffer == [0xFF, 0xD8])
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut non_jpeg_files = Vec::new();

    // First, check all files
    for path in &args.files {
        if !is_jpeg(path)? {
            non_jpeg_files.push(path.clone());
        }
        else{
            // TODO: Extract metadata from the file. For now just print the file path.
            println!("Processing JPEG file: {}", path.display());
        }
    }

    // If there are any non-JPEG files, print error and exit
    if !non_jpeg_files.is_empty() {
        eprintln!("\nThe following files are not valid JPEG images:");
        for path in non_jpeg_files {
            eprintln!("  - {}", path.display());
        }
    }

    Ok(())
}
