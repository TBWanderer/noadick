use bincode::{Decode, Encode};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process;

#[derive(Serialize, Deserialize, Encode, Decode, Debug)]
struct DickOwner {
    name: String,
    size: i16,
    last: i64,
}

type DickOwners = HashMap<i64, DickOwner>;

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        error!("Invalid arguments");
        eprintln!("Usage: {} <input_file.json>", args[0]);
        eprintln!("Converts JSON file to bincode format with MD5-hashed filename");
        process::exit(1);
    }

    let input_path = &args[1];

    if let Err(e) = migrate_file(input_path) {
        error!("Migration failed: {}", e);
        process::exit(1);
    }

    info!("Migration completed successfully!");
}

fn migrate_file(input_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(input_path);

    if !path.exists() {
        return Err(format!("File not found: {}", input_path).into());
    }

    info!("Reading JSON from: {}", input_path);
    let json_content = fs::read_to_string(input_path)?;
    let data: DickOwners =
        serde_json::from_str(&json_content).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    info!("Loaded {} entries", data.len());

    debug!("Encoding to bincode format...");
    let encoded_data = bincode::encode_to_vec(&data, bincode::config::standard())
        .map_err(|e| format!("Failed to encode bincode: {}", e))?;

    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid filename")?;

    let md5_hash = calculate_md5(file_stem);

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let output_path = parent.join(format!("{}.dat", md5_hash));
    let backup_path = parent.join(format!("{}.json.bak", file_stem));

    info!("Creating backup: {}", backup_path.display());
    fs::copy(input_path, &backup_path)?;

    info!("Writing bincode to: {}", output_path.display());
    fs::write(&output_path, &encoded_data)?;

    info!("✓ Original file → {}", backup_path.display());
    info!("✓ Migrated data → {}", output_path.display());
    info!("✓ Filename hash → {} → {}.dat", file_stem, md5_hash);

    fs::remove_file(input_path)?;
    info!("✓ Removed original: {}", input_path);

    Ok(())
}

fn calculate_md5(input: &str) -> String {
    format!("{:x}", md5::compute(input.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md5_calculation() {
        let hash = calculate_md5("5833285630");
        assert_eq!(hash.len(), 32);

        let hash2 = calculate_md5("5833285630");
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_serialization() {
        let mut data = DickOwners::new();
        data.insert(
            1,
            DickOwner {
                name: "Test".to_string(),
                size: 10,
                last: 1234567890,
            },
        );

        let encoded = bincode::encode_to_vec(&data, bincode::config::standard()).unwrap();
        let decoded: DickOwners = bincode::decode_from_slice(&encoded, bincode::config::standard())
            .unwrap()
            .0;

        assert_eq!(data.len(), decoded.len());
    }
}
