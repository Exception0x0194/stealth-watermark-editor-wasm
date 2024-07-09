use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use image::{DynamicImage, GenericImageView, Rgba};
use std::convert::TryInto;
use std::io::{prelude::*, Cursor};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
pub struct DataReader {
    data: Vec<u8>,
    index: usize,
}

#[wasm_bindgen]
impl DataReader {
    #[wasm_bindgen(constructor)]
    pub fn new(data: Vec<u8>) -> DataReader {
        DataReader { data, index: 0 }
    }

    pub fn read_bit(&mut self) -> u8 {
        let bit = self.data[self.index] & 1; // 只读取最低位
        self.index += 1;
        bit
    }

    pub fn read_byte(&mut self) -> u8 {
        let mut byte = 0;
        for i in 0..8 {
            byte |= self.read_bit() << (7 - i);
        }
        byte
    }

    pub fn read_bytes(&mut self, n: usize) -> Vec<u8> {
        (0..n).map(|_| self.read_byte()).collect()
    }

    pub fn read_int32(&mut self) -> i32 {
        let bytes = self.read_bytes(4);
        let bytes4: [u8; 4] = bytes.try_into().unwrap();
        i32::from_be_bytes(bytes4)
    }
}

#[wasm_bindgen]
pub fn embed_stealth_watermark(bytes: Vec<u8>, metadata: String) -> Result<Vec<u8>, JsValue> {
    // Decode
    let img_format = image::guess_format(&bytes)
        .map_err(|e| JsValue::from_str(&format!("Error guessing image format: {}", e)))?;
    let img = image::load_from_memory(&bytes)
        .map_err(|e| JsValue::from_str(&format!("Error decoding image: {}", e)))?;
    let mut imgbuf = img.to_rgba8();

    // Compress metadata with Gzip
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(metadata.as_bytes())
        .map_err(|e| JsValue::from_str(&format!("Error compressing metadata: {}", e)))?;
    let compressed = encoder
        .finish()
        .map_err(|e| JsValue::from_str(&format!("Error finishing compression: {}", e)))?;

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
    DynamicImage::ImageRgba8(imgbuf)
        .write_to(&mut output, img_format)
        .map_err(|e| JsValue::from_str(&format!("Error encoding image: {}", e)))?;

    Ok(output.into_inner())
}

#[wasm_bindgen]
pub fn decode_stealth_watermark(input_bytes: Vec<u8>) -> Result<String, JsValue> {
    let img =
        image::load_from_memory(&input_bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let (width, height) = img.dimensions();

    let mut lowest_data = vec![];

    for x in 0..width {
        for y in 0..height {
            let pixel = img.get_pixel(x, y);
            let a = pixel[3]; // 获取 alpha 值
            lowest_data.push(a & 1);
        }
    }

    let mut reader = DataReader::new(lowest_data);
    let magic = "stealth_pngcomp";
    let magic_string = String::from_utf8(reader.read_bytes(magic.len())).unwrap();

    if magic == magic_string {
        let data_length = reader.read_int32() as usize;
        let gzip_bytes = reader.read_bytes(data_length / 8);
        let mut gz = GzDecoder::new(gzip_bytes.as_slice());
        let mut decompressed_data = String::new();
        gz.read_to_string(&mut decompressed_data)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(decompressed_data)
    } else {
        Err(JsValue::from("Magic number not found"))
    }
}
