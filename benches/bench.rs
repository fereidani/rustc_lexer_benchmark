use std::fs;
use std::hint::black_box;
use std::path::PathBuf;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use rustc_lexer::{Cursor, FrontmatterAllowed};

fn bench_cursor_first(c: &mut Criterion) {
    let input = "fn main() { println!(\"Hello, world!\"); }";
    let mut group = c.benchmark_group("cursor_first");
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("first", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(input), FrontmatterAllowed::No);
            black_box(cursor.first())
        })
    });

    group.finish();
}

fn bench_cursor_iteration(c: &mut Criterion) {
    let input = "fn main() { println!(\"Hello, world!\"); }".repeat(100);
    let mut group = c.benchmark_group("cursor_iteration");
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("bump_all", |b| {
        b.iter(|| {
            let mut cursor = Cursor::new(black_box(&input), FrontmatterAllowed::No);
            while !cursor.is_eof() {
                black_box(cursor.bump());
            }
        })
    });

    group.finish();
}

fn bench_cursor_eat_while(c: &mut Criterion) {
    let input = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa!";
    let mut group = c.benchmark_group("cursor_eat_while");
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("eat_while_alpha", |b| {
        b.iter(|| {
            let mut cursor = Cursor::new(black_box(input), FrontmatterAllowed::No);
            cursor.eat_while(|c| c.is_alphabetic());
            black_box(cursor.first())
        })
    });

    group.finish();
}

fn bench_cursor_eat_until(c: &mut Criterion) {
    let input = "this is a long line of text that ends with a newline\n";
    let mut group = c.benchmark_group("cursor_eat_until");
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("eat_until_newline", |b| {
        b.iter(|| {
            let mut cursor = Cursor::new(black_box(input), FrontmatterAllowed::No);
            cursor.eat_until(b'\n');
            black_box(cursor.first())
        })
    });

    group.finish();
}

/*
fn bench_cursor_peek(c: &mut Criterion) {
    let input = "abcdefghijklmnopqrstuvwxyz";
    let mut group = c.benchmark_group("cursor_peek");

    group.bench_function("first", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(input), FrontmatterAllowed::No);
            black_box(cursor.first())
        })
    });

    group.bench_function("second", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(input), FrontmatterAllowed::No);
            black_box(cursor.second())
        })
    });

    group.bench_function("third", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(input), FrontmatterAllowed::No);
            black_box(cursor.third())
        })
    });

    group.finish();
}
*/

fn bench_strip_shebang(c: &mut Criterion) {
    let input = "#!/bin/bash\necho hello";
    let mut group = c.benchmark_group("strip_shebang");
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("valid_shebang", |b| {
        b.iter(|| black_box(rustc_lexer::strip_shebang(black_box(input))))
    });

    let input_no_shebang = "fn main() {}";
    group.bench_function("no_shebang", |b| {
        b.iter(|| black_box(rustc_lexer::strip_shebang(black_box(input_no_shebang))))
    });

    group.finish();
}

fn bench_tokenize(c: &mut Criterion) {
    let input = "/* my source file */ fn main() { println!(\"zebra\"); }\n";
    let mut group = c.benchmark_group("tokenize");

    group.bench_function("simple_function", |b| {
        b.iter(|| {
            let tokens: Vec<_> =
                rustc_lexer::tokenize(black_box(input), FrontmatterAllowed::No).collect();
            black_box(tokens)
        })
    });

    let lengths = [0usize, 4, 8, 16, 32, 64, 128, 256];
    let mut seed = 12345u32;
    let mut random_ascii = || {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        ((seed % 26) as u8 + b'A') as char
    };
    let mut random_strings = Vec::new();
    let mut rarely_escaped_strings = Vec::new();
    let mut moderately_escaped_strings = Vec::new();
    for &len in &lengths {
        let mut s = String::with_capacity(len + 2);
        s.push('"');
        for _ in 0..len {
            s.push(random_ascii());
        }
        s.push('"');
        random_strings.push(s);
    }
    for &len in &lengths {
        let mut s = String::with_capacity(len + 2);
        s.push('"');
        for i in 0..len {
            let c = if len / 2 == i { '\\' } else { random_ascii() };
            s.push(c);
        }
        s.push('"');
        rarely_escaped_strings.push(s);
    }
    for &len in &lengths {
        // first middle last
        let mut s = String::with_capacity(len * 2 + 2);
        s.push('"');
        for i in 0..len {
            if i == 1 || i == len - 1 {
                s.push('\\')
            }
            if len / 2 == i {
                s.push_str("\\\"");
            } else {
                s.push(random_ascii());
            }
        }
        s.push('"');
        moderately_escaped_strings.push(s);
    }

    // rarely escaped string
    for (i, &len) in lengths.iter().enumerate() {
        if len < 4 {
            continue;
        }
        let input = &rarely_escaped_strings[i];
        group.bench_function(format!("rarely_escaped_string_{}", len), |b| {
            b.iter(|| {
                let mut token_count = 0;
                for token in rustc_lexer::tokenize(black_box(input), FrontmatterAllowed::No) {
                    black_box(token);
                    token_count += 1;
                }
                assert_eq!(token_count, 1);
                black_box(token_count)
            })
        });
    }

    // moderately escaped string
    for (i, &len) in lengths.iter().enumerate() {
        if len < 4 {
            continue;
        }
        let input = &moderately_escaped_strings[i];
        group.bench_function(format!("moderately_escaped_string_{}", len), |b| {
            b.iter(|| {
                let mut token_count = 0;
                for token in rustc_lexer::tokenize(black_box(input), FrontmatterAllowed::No) {
                    black_box(token);
                    token_count += 1;
                }
                assert_eq!(token_count, 1);
                black_box(token_count)
            })
        });
    }

    // Specific tests for each size
    for (i, &len) in lengths.iter().enumerate() {
        let input = &random_strings[i];
        group.bench_function(format!("random_ascii_string_{}", len), |b| {
            b.iter(|| {
                let mut token_count = 0;
                for token in rustc_lexer::tokenize(black_box(input), FrontmatterAllowed::No) {
                    black_box(token);
                    token_count += 1;
                }
                assert_eq!(token_count, 1);
                black_box(token_count)
            })
        });
    }

    let short_strings_input = r#""a""#;
    group.bench_function("short_string", |b| {
        b.iter(|| {
            let mut token_count = 0;
            for token in
                rustc_lexer::tokenize(black_box(short_strings_input), FrontmatterAllowed::No)
            {
                black_box(token);
                token_count += 1;
            }
            assert_eq!(token_count, 1);
            black_box(token_count)
        })
    });

    let long_strings_input = r#""this is a very long string that contains many characters""#;
    group.bench_function("long_strings", |b| {
        b.iter(|| {
            let mut token_count = 0;
            for token in
                rustc_lexer::tokenize(black_box(long_strings_input), FrontmatterAllowed::No)
            {
                black_box(token);
                token_count += 1;
            }
            assert_eq!(token_count, 1);
            black_box(token_count)
        })
    });

    let strings_input = r#"
fn example() {
    let short = "hi";
    let medium = "hello world";
    let long = "this is a much longer string that contains many more characters";
    let very_long = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.";
    let with_escapes = "line1\nline2\ttabbed\r\nwindows line ending";
    let unicode = "خشایار اینجا بود 😊🚀🌍";
    let empty = "";
    let single = "x";
}
"#;
    group.bench_function("strings_in_function", |b| {
        b.iter(|| {
            let tokens: Vec<_> =
                rustc_lexer::tokenize(black_box(strings_input), FrontmatterAllowed::No).collect();
            black_box(tokens)
        })
    });

    let single_line_comments = r"
// line
//// line as well
/// outer doc line
//! inner doc line
";
    group.bench_function("single_line_comments", |b| {
        b.iter(|| {
            let tokens: Vec<_> =
                rustc_lexer::tokenize(black_box(single_line_comments), FrontmatterAllowed::No)
                    .collect();
            black_box(tokens)
        })
    });

    let multi_line_comments = r"
/* block */
/**/
/*** also block */
/** outer doc block */
/*! inner doc block */
/*
This is a
   multiline
   block comment
*/
";
    group.bench_function("multi_line_comments", |b| {
        b.iter(|| {
            let tokens: Vec<_> =
                rustc_lexer::tokenize(black_box(multi_line_comments), FrontmatterAllowed::No)
                    .collect();
            black_box(tokens)
        })
    });

    let literals = r####"
'a'
b'a'
"a"
b"a"
1234
0b101
0xABC
1.0
1.0e10
2us
r###"raw"###suffix
br###"raw"###suffix
"####;
    group.bench_function("literals", |b| {
        b.iter(|| {
            let tokens: Vec<_> =
                rustc_lexer::tokenize(black_box(literals), FrontmatterAllowed::No).collect();
            black_box(tokens)
        })
    });

    group.finish();
}

fn bench_frontmatter(c: &mut Criterion) {
    let input = r#"---cargo
[dependencies]
clap = "4"
---

fn main() {}
"#;
    let mut group = c.benchmark_group("frontmatter");
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("frontmatter_allowed", |b| {
        b.iter(|| {
            let tokens: Vec<_> =
                rustc_lexer::tokenize(black_box(input), FrontmatterAllowed::Yes).collect();
            black_box(tokens)
        })
    });

    group.finish();
}

fn bench_tokenize_real_world(c: &mut Criterion) {
    let home = std::env::var("HOME").expect("HOME not set");
    let toolchain_src = PathBuf::from(home)
        .join(".rustup")
        .join("toolchains")
        .join(if cfg!(target_os = "windows") {
            format!("stable-{}-pc-windows-msvc", std::env::consts::ARCH)
        } else if cfg!(target_os = "macos") {
            format!("stable-{}-apple-darwin", std::env::consts::ARCH)
        } else {
            format!("stable-{}-unknown-linux-gnu", std::env::consts::ARCH)
        })
        .join("lib")
        .join("rustlib")
        .join("src")
        .join("rust")
        .join("library");

    let mut sources: Vec<(String, String)> = Vec::new();
    let mut total_bytes = 0usize;

    fn collect_rs_files(
        dir: &PathBuf,
        sources: &mut Vec<(String, String)>,
        total_bytes: &mut usize,
    ) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect_rs_files(&path, sources, total_bytes);
                } else if path.extension().map_or(false, |ext| ext == "rs") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        *total_bytes += content.len();
                        sources.push((path.display().to_string(), content));
                    }
                }
            }
        }
    }

    collect_rs_files(&toolchain_src, &mut sources, &mut total_bytes);

    if sources.is_empty() {
        eprintln!("Warning: No .rs files found in {:?}", toolchain_src);
        return;
    }

    sources.sort_by(|a, b| a.0.cmp(&b.0));

    println!(
        "Found {} files, {} total",
        sources.len(),
        if total_bytes >= 1_000_000 {
            format!("{:.2} MB", total_bytes as f64 / 1_000_000.0)
        } else if total_bytes >= 1_000 {
            format!("{:.2} KB", total_bytes as f64 / 1_000.0)
        } else {
            format!("{} bytes", total_bytes)
        }
    );

    let mut group = c.benchmark_group("tokenize_real_world");
    group.throughput(Throughput::Bytes(total_bytes as u64));
    group.sample_size(100);

    group.bench_function("stdlib_all_files", |b| {
        b.iter(|| {
            let mut token_count = 0usize;
            for (_path, content) in &sources {
                for token in rustc_lexer::tokenize(black_box(content), FrontmatterAllowed::No) {
                    black_box(token);
                    token_count += 1;
                }
            }
            black_box(token_count)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_tokenize_real_world,
    bench_strip_shebang,
    bench_tokenize,
    bench_frontmatter,
    bench_cursor_first,
    bench_cursor_iteration,
    bench_cursor_eat_while,
    bench_cursor_eat_until,
    //bench_cursor_peek,
);
criterion_main!(benches);
