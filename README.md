# Document Processor

A modern desktop application for parsing and processing documents (PDF, DOCX, TXT) with intelligent image context extraction.

Built with **Rust + Tauri + Svelte** for maximum performance and minimal footprint.

## Features

- **Multi-format parsing**: PDF, DOCX, DOC, TXT, RTF
- **Image extraction with context**: Images are extracted with surrounding text preserved
- **Document classification**: Automatic detection of document types (umowa, pozew, ustawa, etc.)
- **Watch folder**: Automatic processing of new documents
- **SQLite database**: Fast search and organization
- **Modern UI**: Dark theme, responsive design
- **Cross-platform**: Linux and Windows support

## Installation

### Prerequisites

#### Linux (Ubuntu/Debian)
```bash
sudo apt install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libssl-dev
```

#### Windows
- Install [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)
- Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

### Build from source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
cd document-processor
npm install
npm run tauri build
```

### Development

```bash
npm run tauri dev
```

## Usage

### GUI Application

1. Launch the application
2. Drag & drop documents or click to browse
3. Set a "Watch Folder" for automatic processing
4. View processed documents with extracted text and images

### Output Structure

Each processed document creates:
```
processed/<document-id>/
├── document.md        # Human-readable markdown
├── document.json      # Structured data for AI
├── images/
│   ├── img_001.png   # Extracted images
│   ├── img_001.json  # Image metadata + context
│   └── thumb_001.png # Thumbnails
└── original.pdf      # Original file copy
```

### Image Context

Each image includes:
- `context_before`: 200 characters of text before the image
- `context_after`: 200 characters after
- `position_marker`: Page and position reference
- `ocr_text`: Text extracted from image (if applicable)
- `ai_description`: AI-generated description (when available)

## Claude Code Skills

This project includes Claude Code skills for command-line integration:

### /parse
```
/parse ~/Documents/contract.pdf
```
Parses a document and extracts text + images.

### /document-upload-analyzer
```
/document-upload-analyzer
```
Analyzes document upload methods in a web application.

## Architecture

```
document-processor/
├── src/                  # Svelte frontend
│   ├── App.svelte       # Main component
│   ├── main.js          # Entry point
│   └── styles.css       # Global styles
├── src-tauri/
│   └── src/
│       ├── main.rs      # Tauri entry point
│       ├── parser.rs    # Document parsing logic
│       ├── db.rs        # SQLite database
│       └── watcher.rs   # Folder watching
└── package.json
```

## Technologies

- **Backend**: Rust (lopdf, pdf-extract, image, rusqlite)
- **Frontend**: Svelte 5
- **Framework**: Tauri 2
- **Database**: SQLite (rusqlite with bundled)
- **Build**: Vite

## License

Dual-licensed:

- **AGPLv3** for open source, personal, and internal use — see [LICENSE](LICENSE).
- **Commercial license** for SaaS, embedded use, or proprietary modifications — see [LICENSE-COMMERCIAL.md](LICENSE-COMMERCIAL.md).
