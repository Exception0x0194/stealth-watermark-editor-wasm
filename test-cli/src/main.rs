use flate2::write::GzEncoder;
use flate2::Compression;
use image::Rgba;
use std::fs::{self, File};
use std::io::{Cursor, Write};

pub fn embed_stealth_watermark(bytes: Vec<u8>, metadata: String) -> Result<Vec<u8>, String> {
    // Decode
    let img_format = image::guess_format(&bytes)
        .map_err(|e| String::from(&format!("Error guessing image format: {}", e)))?;
    let img = image::load_from_memory(&bytes)
        .map_err(|e| String::from(&format!("Error decoding image: {}", e)))?;
    let mut imgbuf = img.to_rgba8();

    // Compress metadata with Gzip
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(metadata.as_bytes())
        .map_err(|e| String::from(&format!("Error compressing metadata: {}", e)))?;
    let compressed = encoder
        .finish()
        .map_err(|e| String::from(&format!("Error finishing compression: {}", e)))?;

    // Magic numbers
    let magic_bytes = b"stealth_pngcomp";
    let bit_length = (compressed.len() * 8) as u32;
    let mut data_to_embed = Vec::new();
    data_to_embed.extend_from_slice(magic_bytes);
    data_to_embed.extend_from_slice(&bit_length.to_be_bytes());
    data_to_embed.extend_from_slice(&compressed);

    // Write bits
    let mut bit_index = 0;
    for x in 0..imgbuf.width() {
        for y in 0..imgbuf.height() {
            let pixel = imgbuf.get_pixel_mut(x, y);
            if bit_index < data_to_embed.len() * 8 {
                let byte_index = bit_index / 8;
                let bit = (data_to_embed[byte_index] >> (7 - bit_index % 8)) & 1;
                let alpha = pixel[3] & 0xFE | bit as u8;
                *pixel = Rgba([pixel[0], pixel[1], pixel[2], alpha]);
                bit_index += 1;
            } else {
                break;
            }
        }
        if bit_index >= data_to_embed.len() * 8 {
            break;
        }
    }

    // Encode
    let mut output = Cursor::new(Vec::new());
    imgbuf
        .write_to(&mut output, img_format)
        .map_err(|e| String::from(&format!("Error encoding image: {}", e)))?;

    Ok(output.into_inner())
}

fn main() {
    // Input paths
    let input_image_path = "./img/test.png";
    let metadata = "{\"Description\": \"This is some secret data!\"}";
    let output_image_path = "./img/output.png";

    // Read image
    let image_bytes = fs::read(input_image_path).unwrap_or_else(|err| {
        eprintln!("Failed to read image file '{}': {}", input_image_path, err);
        std::process::exit(1);
    });

    // Embed info
    let result = embed_stealth_watermark(image_bytes, metadata.to_string()).unwrap_or_else(|err| {
        eprintln!("Failed to embed watermark: {}", err);
        std::process::exit(1);
    });

    // Write image
    let mut file = File::create(output_image_path).unwrap_or_else(|err| {
        eprintln!(
            "Failed to create output file '{}': {}",
            output_image_path, err
        );
        std::process::exit(1);
    });

    file.write_all(&result).unwrap_or_else(|err| {
        eprintln!(
            "Failed to write to output file '{}': {}",
            output_image_path, err
        );
        std::process::exit(1);
    });

    println!("Watermark embedded successfully!");
}
