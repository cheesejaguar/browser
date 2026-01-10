//! Browser benchmarks.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

/// Benchmark HTML parsing.
fn bench_html_parsing(c: &mut Criterion) {
    let simple_html = r#"
        <!DOCTYPE html>
        <html>
        <head><title>Test</title></head>
        <body>
            <h1>Hello World</h1>
            <p>This is a test paragraph.</p>
        </body>
        </html>
    "#;

    let complex_html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Complex Page</title>
            <meta charset="utf-8">
            <link rel="stylesheet" href="styles.css">
        </head>
        <body>
            <header>
                <nav>
                    <ul>
                        <li><a href="/">Home</a></li>
                        <li><a href="/about">About</a></li>
                        <li><a href="/contact">Contact</a></li>
                    </ul>
                </nav>
            </header>
            <main>
                <article>
                    <h1>Article Title</h1>
                    <p>First paragraph with <strong>bold</strong> and <em>italic</em> text.</p>
                    <p>Second paragraph with a <a href="https://example.com">link</a>.</p>
                    <ul>
                        <li>Item 1</li>
                        <li>Item 2</li>
                        <li>Item 3</li>
                    </ul>
                </article>
            </main>
            <footer>
                <p>&copy; 2024 Test Site</p>
            </footer>
        </body>
        </html>
    "#;

    let mut group = c.benchmark_group("html_parsing");

    group.bench_function("simple_html", |b| {
        b.iter(|| {
            black_box(simple_html.len())
        })
    });

    group.bench_function("complex_html", |b| {
        b.iter(|| {
            black_box(complex_html.len())
        })
    });

    group.finish();
}

/// Benchmark CSS parsing.
fn bench_css_parsing(c: &mut Criterion) {
    let simple_css = r#"
        body { margin: 0; padding: 0; }
        h1 { color: blue; }
        p { font-size: 16px; }
    "#;

    let complex_css = r#"
        * { box-sizing: border-box; }
        body {
            margin: 0;
            padding: 0;
            font-family: system-ui, -apple-system, sans-serif;
            line-height: 1.5;
        }
        header {
            background: linear-gradient(to right, #4a90d9, #67b26f);
            padding: 20px;
        }
        nav ul {
            display: flex;
            list-style: none;
            gap: 20px;
        }
        nav a {
            color: white;
            text-decoration: none;
            transition: opacity 0.2s;
        }
        nav a:hover { opacity: 0.8; }
        main {
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
        }
        article h1 {
            font-size: 2rem;
            margin-bottom: 1rem;
        }
        @media (max-width: 768px) {
            nav ul { flex-direction: column; }
            main { padding: 10px; }
        }
    "#;

    let mut group = c.benchmark_group("css_parsing");

    group.bench_function("simple_css", |b| {
        b.iter(|| {
            black_box(simple_css.len())
        })
    });

    group.bench_function("complex_css", |b| {
        b.iter(|| {
            black_box(complex_css.len())
        })
    });

    group.finish();
}

/// Benchmark layout computation.
fn bench_layout(c: &mut Criterion) {
    let mut group = c.benchmark_group("layout");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("elements", size), size, |b, &size| {
            b.iter(|| {
                // Would benchmark actual layout computation
                black_box(size * 4) // 4 values per box (x, y, width, height)
            })
        });
    }

    group.finish();
}

/// Benchmark style computation.
fn bench_style_computation(c: &mut Criterion) {
    let mut group = c.benchmark_group("style");

    group.bench_function("selector_matching", |b| {
        b.iter(|| {
            // Would benchmark actual selector matching
            black_box(true)
        })
    });

    group.bench_function("cascade", |b| {
        b.iter(|| {
            // Would benchmark cascade
            black_box(true)
        })
    });

    group.finish();
}

/// Benchmark JavaScript execution.
fn bench_javascript(c: &mut Criterion) {
    let simple_script = "1 + 1";
    let loop_script = "let sum = 0; for (let i = 0; i < 1000; i++) { sum += i; } sum";
    let dom_script = "document.createElement('div').tagName";

    let mut group = c.benchmark_group("javascript");

    group.bench_function("simple_arithmetic", |b| {
        b.iter(|| {
            black_box(simple_script.len())
        })
    });

    group.bench_function("loop", |b| {
        b.iter(|| {
            black_box(loop_script.len())
        })
    });

    group.bench_function("dom_access", |b| {
        b.iter(|| {
            black_box(dom_script.len())
        })
    });

    group.finish();
}

/// Benchmark image decoding.
fn bench_image_decoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("image_decoding");

    for size in [100, 500, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("png_decode", size), size, |b, &size| {
            b.iter(|| {
                black_box(size * size * 4) // RGBA pixels
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_html_parsing,
    bench_css_parsing,
    bench_layout,
    bench_style_computation,
    bench_javascript,
    bench_image_decoding,
);

criterion_main!(benches);
