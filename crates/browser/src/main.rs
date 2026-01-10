//! Oxide Browser - A high-performance web browser written in Rust.

use anyhow::Result;
use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use browser::{BrowserConfig, BrowserEngine};

/// Oxide Browser - A high-performance web browser
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// URL to open
    #[arg(default_value = "about:blank")]
    url: String,

    /// Run in headless mode
    #[arg(long)]
    headless: bool,

    /// Disable JavaScript
    #[arg(long)]
    no_javascript: bool,

    /// Disable images
    #[arg(long)]
    no_images: bool,

    /// Viewport width
    #[arg(long, default_value = "1280")]
    width: u32,

    /// Viewport height
    #[arg(long, default_value = "720")]
    height: u32,

    /// Device pixel ratio
    #[arg(long, default_value = "1.0")]
    device_pixel_ratio: f64,

    /// User agent string
    #[arg(long)]
    user_agent: Option<String>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Dump DOM tree
    #[arg(long)]
    dump_dom: bool,

    /// Take screenshot and save to file
    #[arg(long)]
    screenshot: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose { Level::DEBUG } else { Level::INFO };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Oxide Browser v{}", browser::VERSION);
    info!("Starting browser engine...");

    // Build configuration
    let mut config = if args.headless {
        BrowserConfig::headless()
    } else {
        BrowserConfig::default()
    };

    config.viewport_width = args.width;
    config.viewport_height = args.height;
    config.device_pixel_ratio = args.device_pixel_ratio;
    config.javascript_enabled = !args.no_javascript;
    config.images_enabled = !args.no_images;

    if let Some(ua) = args.user_agent {
        config.user_agent = ua;
    }

    // Create and start browser engine
    let engine = BrowserEngine::new(config);
    engine.start();

    // Open the URL
    if args.url != "about:blank" {
        info!("Opening: {}", args.url);
        let page = engine.open_url(&args.url).await?;

        // Wait for load
        while page.is_loading() {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        info!("Page loaded: {}", page.title());

        // Dump DOM if requested
        if args.dump_dom {
            println!("DOM Tree:");
            println!("{}", page.content());
        }

        // Take screenshot if requested
        if let Some(path) = args.screenshot {
            if let Some(data) = page.screenshot() {
                std::fs::write(&path, &data)?;
                info!("Screenshot saved to: {}", path);
            }
        }
    }

    // In non-headless mode, would run the event loop here
    if !args.headless {
        info!("Running in interactive mode...");
        // Would use winit for the window event loop
        // For now, just wait briefly
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    // Cleanup
    engine.stop();
    info!("Browser shutdown complete");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_default() {
        let args = Args::parse_from(["oxide-browser"]);
        assert_eq!(args.url, "about:blank");
        assert!(!args.headless);
        assert!(!args.no_javascript);
    }

    #[test]
    fn test_args_with_url() {
        let args = Args::parse_from(["oxide-browser", "https://example.com"]);
        assert_eq!(args.url, "https://example.com");
    }

    #[test]
    fn test_args_headless() {
        let args = Args::parse_from(["oxide-browser", "--headless"]);
        assert!(args.headless);
    }
}
