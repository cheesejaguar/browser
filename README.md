# Oxide Browser

A modern, high-performance web browser written entirely in Rust.

[![Build](https://github.com/cheesejaguar/browser/actions/workflows/build.yml/badge.svg)](https://github.com/cheesejaguar/browser/actions/workflows/build.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Overview

Oxide Browser is a from-scratch web browser implementation featuring:

- **HTML5 parsing** via html5ever
- **CSS3 parsing and styling** with cascade, inheritance, and selector matching
- **Modern layout engine** supporting block, inline, flexbox, and CSS grid
- **GPU-accelerated rendering** using wgpu (Vulkan/Metal/DX12)
- **JavaScript execution** via the Boa engine
- **Secure networking** with TLS 1.3, HTTP/2, and cookie management
- **Layer-based compositing** for transforms, opacity, and animations

## Supported Platforms

| Platform | Architecture | Status |
|----------|--------------|--------|
| Linux    | x86_64       | Supported |
| Windows  | x86_64       | Supported |
| macOS    | x86_64       | Supported |
| macOS    | aarch64      | Supported |

## Building from Source

### Prerequisites

- Rust 1.75 or later
- System dependencies for your platform:
  - **Linux**: `libxkbcommon-dev`, `libwayland-dev`, `libx11-dev`
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Visual Studio Build Tools

### Build Commands

```bash
# Clone the repository
git clone https://github.com/cheesejaguar/browser.git
cd browser

# Build in debug mode
cargo build

# Build in release mode (recommended for performance)
cargo build --release

# Run the browser
cargo run -p browser --release -- https://example.com
```

### Command Line Options

```bash
oxide-browser [OPTIONS] [URL]

Options:
  --headless        Run without GUI (for testing/scraping)
  --verbose         Enable verbose logging
  --width <WIDTH>   Initial window width (default: 1280)
  --height <HEIGHT> Initial window height (default: 720)
  -h, --help        Print help information
  -V, --version     Print version information
```

## Architecture

Oxide Browser follows a classic multi-stage rendering pipeline:

```
┌─────────┐    ┌─────────┐    ┌────────┐    ┌───────┐    ┌───────────┐
│  Parse  │───▶│  Style  │───▶│ Layout │───▶│ Paint │───▶│ Composite │
└─────────┘    └─────────┘    └────────┘    └───────┘    └───────────┘
```

### Crate Structure

```
crates/
├── browser/        # Main entry point, integrates all components
├── dom/            # DOM tree, nodes, elements, events, Window
├── html_parser/    # HTML5 parsing (html5ever)
├── css_parser/     # CSS parsing (cssparser)
├── style/          # Cascade, inheritance, selector matching
├── layout/         # Box model, block/inline, flexbox, grid
├── render/         # Display list generation, paint operations
├── compositor/     # Layer compositing, transforms, animations
├── gpu/            # Hardware-accelerated rendering (wgpu)
├── networking/     # HTTP client, TLS, cookies, resource loading
├── js_engine/      # JavaScript execution (Boa) with DOM bindings
├── web_apis/       # Web APIs (Fetch, Storage, WebSocket, etc.)
├── ui/             # Browser chrome, tabs, DevTools
├── media/          # Media playback support
├── security/       # Security features
├── cache/          # Resource caching
└── common/         # Shared utilities and types
```

### Key Dependencies

| Category | Libraries |
|----------|-----------|
| Parsing | html5ever, cssparser, selectors |
| Graphics | wgpu, winit, softbuffer, fontdue |
| Networking | reqwest, hyper, rustls, tokio |
| JavaScript | boa_engine |
| Data Structures | indexmap, slotmap, smallvec |

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p dom
cargo test -p layout

# Run a specific test
cargo test -p browser test_name
```

### Running Benchmarks

```bash
cargo bench -p browser
```

### Code Quality

```bash
# Check without building
cargo check

# Run lints
cargo clippy

# Format code
cargo fmt
```

## Features Roadmap

- [x] HTML5 parsing
- [x] CSS3 parsing and cascade
- [x] Block and inline layout
- [x] Flexbox layout
- [x] CSS Grid layout
- [x] GPU-accelerated rendering
- [x] Basic JavaScript execution
- [ ] Full Web API coverage
- [ ] WebAssembly support
- [ ] Service Workers
- [ ] IndexedDB
- [ ] WebRTC

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

Oxide Browser builds upon the excellent work of the Rust ecosystem, including:

- [Servo](https://servo.org/) - Many architectural inspirations
- [html5ever](https://github.com/servo/html5ever) - HTML parsing
- [cssparser](https://github.com/servo/rust-cssparser) - CSS parsing
- [wgpu](https://wgpu.rs/) - Cross-platform graphics
- [Boa](https://boajs.dev/) - JavaScript engine
