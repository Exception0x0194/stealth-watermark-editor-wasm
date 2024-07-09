# Stealth Watermark Editor

A wasm pack for embedding and reading stealth watermarks in pictures' alpha channels, inspired by techniques discussed in [NovelAI's GitHub repo](https://github.com/NovelAI/novelai-image-metadata).

## Installation

```bash
npm install stealth-watermark-editor
```

## Usage

Below are examples of how to embed and decode information into/from an image's alpha channel:

### Embedding Watermark

```javascript
import init, { embed_stealth_watermark } from 'stealth-watermark-editor';

// Initialize the WASM module only once
await init();

try {
    const imgSrc = 'path/to/image.png'; // Path to the image file
    const metadata = '{"Description": "JSON metadata"}'; // Metadata to embed
    const response = await fetch(imgSrc);
    const imageBuffer = await response.arrayBuffer();

    const watermarkedImage = await embed_stealth_watermark(new Uint8Array(imageBuffer), metadata);
    console.log('Watermark embedded successfully');
} catch (error) {
    console.error('Failed to embed watermark:', error);
}
```

### Decoding Watermark

```javascript
import init, { decode_stealth_watermark } from 'stealth-watermark-editor';

// Initialize the WASM module only once
await init();

try {
    const response = await fetch('path/to/watermarked_image.png');
    const imageBuffer = await response.arrayBuffer();
    const result = await decode_stealth_watermark(new Uint8Array(imageBuffer));
    console.log('Decoded Metadata:', result);
} catch (error) {
    console.error('Failed to decode watermark:', error);
}
```

### API Reference

- `embed_stealth_watermark(imageBytes: Uint8Array, metadata: string): Promise<Uint8Array>`
  - **Parameters**: 
    - `imageBytes` - A `Uint8Array` containing the bytes of the image.
    - `metadata` - A string containing the metadata to embed.
  - **Returns**: A `Uint8Array` containing the bytes of the image with embedded watermark.
  - **Throws**: An error if the watermark cannot be embedded.

- `decode_stealth_watermark(imageBytes: Uint8Array): Promise<string>`
  - **Parameters**: 
    - `imageBytes` - A `Uint8Array` containing the bytes of the watermarked image.
  - **Returns**: A string containing the decoded information.
  - **Throws**: An error if the watermark cannot be decoded.

### Supported Image Formats

The functions support images with an alpha channel such as `.png` and `.webp`, as long as Rust's image library can decode them.