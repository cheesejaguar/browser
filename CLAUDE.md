# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Oxide Browser is a web browser written in Rust using a multi-crate workspace architecture. The browser binary is named `oxide-browser`.

## Build and Development Commands

```bash
# Build the entire workspace
cargo build

# Build in release mode (with LTO enabled)
cargo build --release

# Run the browser
cargo run -p browser -- [URL] [OPTIONS]

# Run with verbose logging
cargo run -p browser -- --verbose https://example.com

# Run in headless mode
cargo run -p browser -- --headless https://example.com

# Run tests for all crates
cargo test

# Run tests for a specific crate
cargo test -p dom
cargo test -p html_parser

# Run a specific test
cargo test -p browser test_args_default

# Run benchmarks
cargo bench -p browser

# Check all crates without building
cargo check

# Run clippy lints
cargo clippy
```

## Architecture

The browser follows a classic multi-stage rendering pipeline: **Parse -> Style -> Layout -> Paint -> Composite**

### Core Crates

- **browser** (`crates/browser`) - Main entry point, integrates all components. Contains `BrowserEngine`, `Page`, and `RenderPipeline`.
- **dom** (`crates/dom`) - DOM tree implementation with `Node`, `Document`, `Element`, event handling, and `Window` object.
- **html_parser** (`crates/html_parser`) - HTML5 parsing using html5ever, converts HTML to DOM.
- **css_parser** (`crates/css_parser`) - CSS parsing using cssparser. Handles stylesheets, selectors, and property declarations.
- **style** (`crates/style`) - CSS cascade, inheritance, selector matching, and style resolution via `Stylist`.
- **layout** (`crates/layout`) - Box model, block/inline formatting, flexbox, and grid layout.
- **render** (`crates/render`) - Display list generation, text rasterization, and paint operations.
- **compositor** (`crates/compositor`) - Layer-based compositing for transforms, opacity, filters, and animations.
- **gpu** (`crates/gpu`) - Hardware-accelerated rendering backend using wgpu.

### Supporting Crates

- **networking** (`crates/networking`) - HTTP client, TLS, connection pooling, cookies, resource loading.
- **js_engine** (`crates/js_engine`) - JavaScript execution using Boa engine with DOM bindings and event loop.
- **web_apis** (`crates/web_apis`) - Web API implementations (Fetch, WebSocket, Storage, History, etc.).
- **ui** (`crates/ui`) - Browser chrome: window, tabs, address bar, navigation controls, DevTools.
- **media** (`crates/media`) - Media playback support.
- **security** (`crates/security`) - Security features.
- **cache** (`crates/cache`) - Caching layer.
- **common** (`crates/common`) - Shared utilities.

### Render Pipeline Flow

The `RenderPipeline` in `browser/src/pipeline.rs` orchestrates rendering through five stages:
1. **Parse** - HTML/CSS parsing into DOM and stylesheets
2. **Style** - Compute styles for each DOM node
3. **Layout** - Calculate box dimensions and positions
4. **Paint** - Generate display list of draw commands
5. **Composite** - Combine layers for final output

### Key Dependencies

- **Parsing**: html5ever, cssparser, selectors
- **Graphics**: wgpu, winit, softbuffer, fontdue
- **Networking**: reqwest, hyper, rustls
- **JavaScript**: boa_engine
- **Async**: tokio
