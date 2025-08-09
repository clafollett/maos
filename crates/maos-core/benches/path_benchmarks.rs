//! Performance benchmarks for path utilities
//!
//! This benchmark suite measures the performance of core path operations
//! to ensure they meet performance requirements for high-frequency usage.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use maos_core::path::{PathValidator, normalize_path, paths_equal, relative_path};
use maos_core::{AgentType, SessionId};
use std::path::{Path, PathBuf};

/// Benchmark normalize_path function with various path complexities
fn bench_normalize_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("normalize_path");

    let test_cases = vec![
        ("simple", "file.txt"),
        ("current_dir", "./file.txt"),
        ("parent_dir", "../file.txt"),
        ("nested", "dir1/dir2/dir3/file.txt"),
        ("complex_traversal", "./dir1/../dir2/./dir3/../file.txt"),
        ("deep_traversal", "a/b/c/d/e/../../../f/g/h/../i.txt"),
        ("windows_separators", "dir1\\dir2\\file.txt"),
        ("mixed_separators", "dir1/dir2\\dir3/file.txt"),
        ("unicode_path", "测试/файл/📁/file.txt"),
    ];

    for (name, path_str) in test_cases {
        group.bench_with_input(BenchmarkId::new("path", name), &path_str, |b, &path_str| {
            let path = Path::new(path_str);
            b.iter(|| normalize_path(black_box(path)))
        });
    }

    // Separate benchmark for long path to avoid borrowing issues
    let long_path = format!("{}file.txt", "very_long_directory_name/".repeat(20));
    group.bench_with_input(
        BenchmarkId::new("path", "long_path"),
        &long_path,
        |b, path_str| {
            let path = Path::new(path_str);
            b.iter(|| normalize_path(black_box(path)))
        },
    );

    group.finish();
}

/// Benchmark paths_equal function
fn bench_paths_equal(c: &mut Criterion) {
    let mut group = c.benchmark_group("paths_equal");

    let test_cases = vec![
        ("identical", ("file.txt", "file.txt")),
        ("normalized_vs_raw", ("./file.txt", "file.txt")),
        ("traversal_comparison", ("dir/../file.txt", "file.txt")),
        ("different_separators", ("dir\\file.txt", "dir/file.txt")),
        (
            "complex_paths",
            ("./a/b/../c/./d/file.txt", "a/c/d/file.txt"),
        ),
        ("absolute_vs_relative", ("/tmp/file.txt", "tmp/file.txt")),
    ];

    for (name, (path1_str, path2_str)) in test_cases {
        group.bench_with_input(
            BenchmarkId::new("comparison", name),
            &(path1_str, path2_str),
            |b, &(path1_str, path2_str)| {
                let path1 = Path::new(path1_str);
                let path2 = Path::new(path2_str);
                b.iter(|| paths_equal(black_box(path1), black_box(path2)))
            },
        );
    }

    // Separate benchmark for long paths
    let long_path = format!("{}file.txt", "long_dir/".repeat(50));
    group.bench_with_input(
        BenchmarkId::new("comparison", "long_paths"),
        &long_path,
        |b, path_str| {
            let path1 = Path::new(path_str);
            let path2 = Path::new(path_str);
            b.iter(|| paths_equal(black_box(path1), black_box(path2)))
        },
    );

    group.finish();
}

/// Benchmark relative_path function
fn bench_relative_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("relative_path");

    let test_cases = vec![
        ("same_dir", ("/base", "/base")),
        ("direct_child", ("/base", "/base/child")),
        ("parent", ("/base/child", "/base")),
        ("sibling", ("/base/dir1", "/base/dir2")),
        ("deep_nested", ("/a/b/c/d", "/a/b/e/f/g")),
        ("different_roots", ("/usr/local", "/var/log")),
        (
            "complex_paths",
            (
                "/home/user/projects/maos/src",
                "/home/user/documents/data/file.txt",
            ),
        ),
    ];

    for (name, (base_str, target_str)) in test_cases {
        group.bench_with_input(
            BenchmarkId::new("relative", name),
            &(base_str, target_str),
            |b, &(base_str, target_str)| {
                let base = Path::new(base_str);
                let target = Path::new(target_str);
                b.iter(|| relative_path(black_box(base), black_box(target)))
            },
        );
    }

    group.finish();
}

/// Benchmark PathValidator construction
fn bench_path_validator_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("path_validator_construction");

    let temp_dir = std::env::temp_dir();

    // Test different numbers of allowed roots and blocked patterns
    let construction_scenarios = vec![
        ("empty", (0, 0)),
        ("single_root", (1, 0)),
        ("multiple_roots", (5, 0)),
        ("with_patterns", (1, 10)),
        ("complex", (5, 20)),
    ];

    for (name, (num_roots, num_patterns)) in construction_scenarios {
        group.bench_with_input(
            BenchmarkId::new("construct", name),
            &(num_roots, num_patterns),
            |b, &(num_roots, num_patterns)| {
                b.iter(|| {
                    let roots = (0..num_roots)
                        .map(|i| temp_dir.join(format!("root_{}", i)))
                        .collect();
                    let patterns = (0..num_patterns)
                        .map(|i| format!("**/*pattern_{}.tmp", i))
                        .collect();

                    PathValidator::new(black_box(roots), black_box(patterns))
                })
            },
        );
    }

    group.finish();
}

/// Benchmark PathValidator validation operations
fn bench_path_validator_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("path_validator_operations");

    let temp_dir = std::env::temp_dir();
    let allowed_roots = vec![temp_dir.clone()];
    let blocked_patterns = vec![
        "*.tmp".to_string(),
        "*.log".to_string(),
        "**/.git/**".to_string(),
        "**/node_modules/**".to_string(),
        "**/.ssh/**".to_string(),
    ];
    let validator = PathValidator::new(allowed_roots, blocked_patterns);

    let validation_scenarios = vec![
        ("simple_valid", "data.txt"),
        ("nested_valid", "project/src/main.rs"),
        ("blocked_extension", "temp.tmp"),
        ("blocked_directory", ".git/config"),
        ("deep_nested", "very/deep/nested/path/to/file.txt"),
        ("traversal_attempt", "../../../etc/passwd"),
        ("unicode_attack", "..\\u{FF0F}etc\\u{2044}passwd"),
        ("url_encoded", "%2e%2e/%2e%2e/etc/passwd"),
        ("control_chars", "safe.txt\0../../../etc/passwd"),
    ];

    for (name, path_str) in validation_scenarios {
        group.bench_with_input(
            BenchmarkId::new("validate", name),
            &path_str,
            |b, &path_str| {
                let path = PathBuf::from(path_str);
                b.iter(|| validator.validate_workspace_path(black_box(&path), black_box(&temp_dir)))
            },
        );
    }

    // Separate benchmark for long path
    let long_path = format!("{}file.txt", "deeply_nested/".repeat(50));
    group.bench_with_input(
        BenchmarkId::new("validate", "long_path"),
        &long_path,
        |b, path_str| {
            let path = PathBuf::from(path_str);
            b.iter(|| validator.validate_workspace_path(black_box(&path), black_box(&temp_dir)))
        },
    );

    group.finish();
}

/// Benchmark blocked path checking
fn bench_blocked_path_checking(c: &mut Criterion) {
    let mut group = c.benchmark_group("blocked_path_checking");

    let blocked_patterns = vec![
        "*.tmp".to_string(),
        "*.log".to_string(),
        "*.bak".to_string(),
        "**/.git/**".to_string(),
        "**/node_modules/**".to_string(),
        "**/.ssh/**".to_string(),
        "**/target/**".to_string(),
        "**/*.o".to_string(),
        "**/*.so".to_string(),
        "**/.DS_Store".to_string(),
    ];
    let validator = PathValidator::new(vec![], blocked_patterns);

    let blocking_scenarios = vec![
        ("allowed_rust", "src/main.rs"),
        ("allowed_config", "config.toml"),
        ("blocked_tmp", "temp.tmp"),
        ("blocked_git", ".git/config"),
        ("blocked_node_modules", "node_modules/react/index.js"),
        ("blocked_nested", "deep/path/to/.ssh/id_rsa"),
        ("blocked_object", "target/debug/main.o"),
        ("complex_allowed", "project/src/components/ui/button.tsx"),
        ("complex_blocked", "project/target/debug/deps/lib.so"),
        ("edge_case", "legitimate_file.tmp.backup"),
    ];

    for (name, path_str) in blocking_scenarios {
        group.bench_with_input(
            BenchmarkId::new("block_check", name),
            &path_str,
            |b, &path_str| {
                let path = PathBuf::from(path_str);
                b.iter(|| validator.is_blocked_path(black_box(&path)))
            },
        );
    }

    group.finish();
}

/// Benchmark workspace path generation
fn bench_workspace_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("workspace_generation");

    let validator = PathValidator::new(vec![], vec![]);
    let base_path = PathBuf::from("/workspaces");

    let generation_scenarios = vec![
        ("short_names", "frontend"),
        ("long_names", "very_long_agent_name_backend_engineer"),
        ("special_chars", "agent-with-dashes_and_underscores"),
        ("unicode_safe", "测试_agent"),
    ];

    for (name, agent_name) in generation_scenarios {
        group.bench_with_input(
            BenchmarkId::new("generate", name),
            &agent_name,
            |b, &agent_name| {
                b.iter(|| {
                    let session_id = SessionId::generate();
                    let agent_type: AgentType = agent_name.to_string();
                    validator.generate_workspace_path(
                        black_box(&base_path),
                        black_box(&session_id),
                        black_box(&agent_type),
                    )
                })
            },
        );
    }

    group.finish();
}

/// Benchmark security pattern detection
fn bench_security_pattern_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("security_detection");

    let temp_dir = std::env::temp_dir();
    let validator = PathValidator::new(vec![temp_dir.clone()], vec![]);

    let security_scenarios = vec![
        ("clean_path", "normal/file.txt"),
        ("basic_traversal", "../../../etc/passwd"),
        ("unicode_traversal", "..\\u{FF0F}etc\\u{2044}passwd"),
        ("url_encoded", "%2e%2e/%2e%2e/etc/passwd"),
        ("double_encoded", "%252e%252e/%252e%252e/etc/passwd"),
        ("null_injection", "safe.txt\0../../../etc/passwd"),
        ("newline_injection", "safe.txt\n../../../etc/passwd"),
        ("mixed_attack", "..\\\\..\\u{FF0F}%2e%2e/etc/passwd"),
        (
            "complex_unicode",
            "测试\\u{FF0F}..\\u{2044}файл\\u{2215}etc/passwd",
        ),
    ];

    for (name, path_str) in security_scenarios {
        group.bench_with_input(
            BenchmarkId::new("detect", name),
            &path_str,
            |b, &path_str| {
                let path = PathBuf::from(path_str);
                b.iter(|| validator.validate_workspace_path(black_box(&path), black_box(&temp_dir)))
            },
        );
    }

    // Separate benchmark for long traversal path
    let long_traversal = format!("{}etc/passwd", "../".repeat(100));
    group.bench_with_input(
        BenchmarkId::new("detect", "long_traversal"),
        &long_traversal,
        |b, path_str| {
            let path = PathBuf::from(path_str);
            b.iter(|| validator.validate_workspace_path(black_box(&path), black_box(&temp_dir)))
        },
    );

    group.finish();
}

criterion_group!(
    benches,
    bench_normalize_path,
    bench_paths_equal,
    bench_relative_path,
    bench_path_validator_construction,
    bench_path_validator_operations,
    bench_blocked_path_checking,
    bench_workspace_generation,
    bench_security_pattern_detection
);

criterion_main!(benches);
