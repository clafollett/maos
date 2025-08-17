//! Test that normalize_path handles invalid and cross-platform paths correctly

use maos_core::path::normalize_path;
use std::path::PathBuf;

#[test]
fn test_windows_paths_on_unix() {
    // On Unix, Windows paths should NOT be treated as absolute
    if !cfg!(windows) {
        let windows_paths = vec![
            ("C:\\Windows\\System32", false),
            ("D:/Program Files", false),
            ("\\\\server\\share", true), // This might become /server/share after normalization
            ("C:", false),
            ("E:\\", false),
        ];

        for (path_str, might_be_absolute) in windows_paths {
            let path = PathBuf::from(path_str);
            let normalized = normalize_path(&path);

            println!(
                "Unix handling Windows path: '{path_str}' -> '{normalized:?}' (absolute={})",
                normalized.is_absolute()
            );

            if !might_be_absolute {
                assert!(
                    !normalized.is_absolute(),
                    "Windows path '{path_str}' should NOT be absolute on Unix, but got {normalized:?}"
                );
            }
        }
    }
}

#[test]
fn test_unix_paths_on_windows() {
    // On Windows, Unix paths should NOT be treated as absolute
    if cfg!(windows) {
        let unix_paths = vec![
            ("/usr/bin", false),
            ("/etc/passwd", false),
            ("/home/user/.config", false),
        ];

        for (path_str, _) in unix_paths {
            let path = PathBuf::from(path_str);
            let normalized = normalize_path(&path);

            println!(
                "Windows handling Unix path: '{path_str}' -> '{normalized:?}' (absolute={})",
                normalized.is_absolute()
            );

            assert!(
                !normalized.is_absolute(),
                "Unix path '{path_str}' should NOT be absolute on Windows, but got {normalized:?}"
            );
        }
    }
}

#[test]
fn test_malicious_paths_are_sanitized() {
    let malicious_paths = vec![
        "../../../etc/passwd",
        "..\\..\\..\\Windows\\System32",
        "test\u{FF0F}file",           // Unicode fullwidth solidus
        "path\u{2044}to\u{2044}file", // Fraction slash
        "./../.././../etc/shadow",
        "",
        ".",
        "..",
    ];

    for path_str in malicious_paths {
        let path = PathBuf::from(path_str);

        // Should not panic - normalize_path must handle anything
        let normalized = normalize_path(&path);

        println!(
            "Malicious path: '{path_str}' -> '{normalized:?}' (absolute={})",
            normalized.is_absolute()
        );

        // None of these should be absolute (they're all relative or malicious)
        assert!(
            !normalized.is_absolute() || path_str.is_empty(),
            "Malicious path '{path_str}' was treated as absolute: {normalized:?}"
        );
    }
}

#[test]
fn test_normalize_preserves_platform_semantics() {
    // Test that paths valid on THIS platform remain valid
    if cfg!(windows) {
        let valid_paths = vec!["C:\\Users\\test", "D:/Program Files/app.exe"];

        for path_str in valid_paths {
            let path = PathBuf::from(path_str);
            let normalized = normalize_path(&path);

            assert!(
                normalized.is_absolute(),
                "Valid Windows path '{path_str}' should remain absolute, got {normalized:?}"
            );
        }
    } else {
        let valid_paths = vec!["/usr/local/bin", "/home/user/.config"];

        for path_str in valid_paths {
            let path = PathBuf::from(path_str);
            let normalized = normalize_path(&path);

            assert!(
                normalized.is_absolute(),
                "Valid Unix path '{path_str}' should remain absolute, got {normalized:?}"
            );
        }
    }
}

#[test]
fn test_our_security_transforms() {
    // Our normalize_path converts backslashes to forward slashes as part of security
    // This test verifies the behavior
    let path_with_backslash = PathBuf::from("some\\path\\file");
    let normalized = normalize_path(&path_with_backslash);

    // After normalization, backslashes should be converted to forward slashes
    let normalized_str = normalized.to_string_lossy();
    assert!(
        !normalized_str.contains('\\') || cfg!(windows),
        "Backslashes should be converted to forward slashes (except on Windows): {normalized:?}"
    );

    // Unicode slashes should definitely be converted
    let unicode_path = PathBuf::from("test\u{FF0F}file");
    let normalized_unicode = normalize_path(&unicode_path);
    let normalized_unicode_str = normalized_unicode.to_string_lossy();

    assert!(
        !normalized_unicode_str.contains('\u{FF0F}'),
        "Unicode slashes should be converted to regular slashes: {normalized_unicode:?}"
    );
}
