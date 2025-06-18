use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use clap::Parser;
use exif::{Reader, Tag, In};
use serde::Serialize;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct FilesystemMetadata {
    size: u64,
    created_time: DateTime<Utc>,
    modified_time: DateTime<Utc>,
}

#[derive(Debug)]
struct ExifMetadata {
    orientation: Option<u32>,
    capture_time: Option<DateTime<Utc>>,
    camera_model: Option<String>,
    camera_serial: Option<String>,
}

/// Command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // JPEG image files to process
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

/// Metadata extracted from a JPEG image
#[derive(Debug, Serialize)]
struct ImageMetadata {
    filename: String,
    size: u64,
    created_time: DateTime<Utc>,
    modified_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    orientation: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    capture_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    camera_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    camera_serial: Option<String>,
}

/// Extract filesystem metadata from a file
fn extract_filesystem_metadata(path: &Path) -> Result<FilesystemMetadata> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("Failed to read metadata for {}", path.display()))?;

    let created_time = metadata.created()
        .with_context(|| format!("Failed to get creation time for {}", path.display()))?;
    let modified_time = metadata.modified()
        .with_context(|| format!("Failed to get modification time for {}", path.display()))?;

    Ok(FilesystemMetadata {
        size: metadata.len(),
        created_time: DateTime::from(created_time),
        modified_time: DateTime::from(modified_time),
    })
}

/// Extract EXIF metadata from a JPEG file
fn extract_exif_metadata(path: &Path) -> Result<ExifMetadata> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("Failed to open file {}", path.display()))?;
    
    let mut bufreader = std::io::BufReader::new(file);
    let exifreader = Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader)?;

    let orientation = exif.get_field(Tag::Orientation, In::PRIMARY)
        .and_then(|field| field.value.get_uint(0));

    let capture_time = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY)
        .and_then(|field| {
            let s = field.display_value().with_unit(&exif).to_string();
            chrono::NaiveDateTime::parse_from_str(&s, "%Y:%m:%d %H:%M:%S").ok()
                .map(|dt| Utc.from_utc_datetime(&dt))
        });

    let camera_model = exif.get_field(Tag::Model, In::PRIMARY)
        .map(|field| field.display_value().with_unit(&exif).to_string());

    let camera_serial = exif.get_field(Tag::BodySerialNumber, In::PRIMARY)
        .map(|field| field.display_value().with_unit(&exif).to_string());

    Ok(ExifMetadata {
        orientation,
        capture_time,
        camera_model,
        camera_serial,
    })
}

/// Process a single JPEG file and generate its metadata JSON
fn process_file(path: &Path) -> Result<()> {
    let fs_metadata = extract_filesystem_metadata(path)?;
    let exif_metadata = extract_exif_metadata(path)?;

    let metadata = ImageMetadata {
        filename: path.file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?
            .to_string(),
        size: fs_metadata.size,
        created_time: fs_metadata.created_time,
        modified_time: fs_metadata.modified_time,
        orientation: exif_metadata.orientation,
        capture_time: exif_metadata.capture_time,
        camera_model: exif_metadata.camera_model,
        camera_serial: exif_metadata.camera_serial,
    };

    // Create output path by replacing extension with .json
    let output_path: PathBuf = path.with_extension("json");
    
    // Write JSON to file
    let json: String = serde_json::to_string_pretty(&metadata)?;
    fs::write(&output_path, json)
        .with_context(|| format!("Failed to write metadata to {}", output_path.display()))?;

    println!("Processed: {}", path.display());
    Ok(())
}

/// Checks if the file is a valid JPEG
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

    // Check if the files are valid JPEG images and extract metadata from the valid ones
    for path in &args.files {
        if !path.exists() {
            continue;
        }
        if !is_jpeg(path)? {
            non_jpeg_files.push(path.clone());
        }
        else if let Err(e) = process_file(&path) {
            eprintln!("Error processing {}: {}", path.display(), e);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_is_jpeg_true() {
        let path = PathBuf::from("images/JAM26284.jpg");
        assert_eq!(is_jpeg(&path).unwrap(), true);
    }

    #[test]
    fn test_is_jpeg_false() {
        let path = PathBuf::from("images/non-jpeg.png");
        assert_eq!(is_jpeg(&path).unwrap(), false);
    }

    #[test]
    fn test_extract_filesystem_metadata() {
        let path = PathBuf::from("images/JAM19896.jpg");
        let meta = extract_filesystem_metadata(&path).unwrap();
        assert!(meta.size == 3014190);
        
        let expected_time = Utc.with_ymd_and_hms(2020, 8, 13, 10, 57, 7).unwrap();
        assert_eq!(meta.created_time, expected_time);
        assert_eq!(meta.modified_time, expected_time);
    }

    #[test]
    fn test_extract_exif_metadata() {
        let path = PathBuf::from("images/JAM26284.jpg");
        let exif = extract_exif_metadata(&path).unwrap();
        assert_eq!(exif.orientation, Some(1));
        assert_eq!(exif.camera_model, Some("\"Canon EOS 5D Mark IV\"".to_string()));
        assert_eq!(exif.camera_serial, Some("\"025021000535\"".to_string()));
    }

    #[test]
    fn test_process_file() {
        let path = PathBuf::from("images/JAM26284.jpg");
        // Should not panic or error
        assert!(process_file(&path).is_ok());
        // Optionally, check that the output JSON file was created
        let json_path = path.with_extension("json");
        assert!(json_path.exists());
    }
} 