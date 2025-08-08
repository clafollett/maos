use maos_core::error::{
    ConfigError, ErrorContext, ExitCode, FileSystemError, GitError, IntoMaosError, MaosError,
    Result, SecurityError, SessionError, ValidationError,
};

#[test]
fn test_invalid_input_error_and_exit_code() {
    let err = MaosError::InvalidInput {
        message: "bad arg".into(),
    };
    let code: ExitCode = (&err).into();
    assert_eq!(code as i32, ExitCode::GeneralError as i32);
    let _res: Result<()> = Err(err);
}

#[test]
fn test_blocking_exit_code_mapping() {
    let err = MaosError::Blocking {
        reason: "policy".into(),
    };
    let code: ExitCode = (&err).into();
    assert_eq!(code, ExitCode::BlockingError);
}

#[test]
fn test_validation_error_variants() {
    // Test structured validation errors
    let val = ValidationError::RequiredFieldMissing {
        field: "username".into(),
    };
    let err: MaosError = val.into();
    let code: ExitCode = (&err).into();
    assert_eq!(code, ExitCode::GeneralError);

    let val2 = ValidationError::OutOfRange {
        field: "age".into(),
        value: "150".into(),
        min: "0".into(),
        max: "120".into(),
    };
    let display = format!("{val2}");
    assert!(display.contains("out of range"));
    assert!(display.contains("age"));
}

#[test]
fn test_error_context_preserved() {
    fn might_fail() -> Result<()> {
        Err(ValidationError::RequiredFieldMissing {
            field: "name".into(),
        })
        .into_maos_error()
        .with_context(|| "while parsing config".to_string())
    }

    let err = might_fail().unwrap_err();
    let s = format!("{err}");
    assert!(s.contains("while parsing config"));
    assert!(s.contains("Required field missing"));
}

#[test]
fn test_nested_context_exit_codes() {
    // Test that nested contexts preserve the original error's exit code
    let sec_err = SecurityError::Unauthorized {
        resource: "secret".into(),
    };
    // First convert to MaosError, then wrap in context
    let maos_err: MaosError = sec_err.into();
    let with_context = MaosError::Context {
        message: "during startup".into(),
        source: Box::new(maos_err) as Box<dyn std::error::Error + Send + Sync>,
    };

    // Should still map to SecurityError exit code
    let code: ExitCode = (&with_context).into();
    assert_eq!(code, ExitCode::SecurityError);
}

#[test]
fn test_anyhow_integration() {
    // Test that we can wrap anyhow errors
    let anyhow_err = anyhow::anyhow!("external library error");
    let maos_err: MaosError = anyhow_err.into();
    let display = format!("{maos_err}");
    assert!(display.contains("external library error"));

    // Anyhow errors should map to InternalError (exit code 99)
    let code: ExitCode = (&maos_err).into();
    assert_eq!(code, ExitCode::InternalError);
}

#[test]
fn test_timeout_exit_code_mapping() {
    let err = MaosError::Timeout {
        operation: "op".into(),
        timeout_ms: 1234,
    };
    let code: ExitCode = (&err).into();
    assert_eq!(code, ExitCode::TimeoutError);
}

#[test]
fn test_from_std_io_error() {
    let io_err = std::io::Error::other("oops");
    let err: MaosError = io_err.into();
    // Default mapping for IO should be GeneralError
    let code: ExitCode = (&err).into();
    assert_eq!(code, ExitCode::GeneralError);
}

#[test]
fn test_config_error_variants() {
    let cfg_err = ConfigError::FileNotFound {
        path: "/etc/maos/config.toml".into(),
    };
    let err: MaosError = cfg_err.into();
    let code: ExitCode = (&err).into();
    assert_eq!(code, ExitCode::ConfigError);

    let cfg_err2 = ConfigError::InvalidValue {
        field: "timeout".into(),
        value: "-1".into(),
        reason: "must be positive".into(),
    };
    let display = format!("{cfg_err2}");
    assert!(display.contains("timeout"));
    assert!(display.contains("-1"));
}

#[test]
fn test_session_error_variants() {
    let sess_err = SessionError::NotFound {
        id: "sess_123".into(),
    };
    let display = format!("{sess_err}");
    assert!(display.contains("not found"));
    assert!(display.contains("sess_123"));

    let sess_err2 = SessionError::InvalidStateTransition {
        from: "active".into(),
        to: "expired".into(),
    };
    let err: MaosError = sess_err2.into();
    let display = format!("{err}");
    assert!(display.contains("state transition"));
}

#[test]
fn test_security_error_variants() {
    let sec_err = SecurityError::PathTraversal {
        path: "../../../etc/passwd".into(),
    };
    let err: MaosError = sec_err.into();
    let code: ExitCode = (&err).into();
    assert_eq!(code, ExitCode::SecurityError);

    let sec_err2 = SecurityError::Unauthorized {
        resource: "/admin/panel".into(),
    };
    let display = format!("{sec_err2}");
    assert!(display.contains("Unauthorized"));
}

#[test]
fn test_filesystem_error_variants() {
    let fs_err = FileSystemError::NotFound {
        path: "/tmp/missing.txt".into(),
    };
    let display = format!("{fs_err}");
    assert!(display.contains("not found"));

    let fs_err2 = FileSystemError::NoSpace;
    let display = format!("{fs_err2}");
    assert!(display.contains("space"));
}

#[test]
fn test_git_error_variants() {
    let git_err = GitError::NotARepository;
    let display = format!("{git_err}");
    assert!(display.to_lowercase().contains("not a git repository"));

    let git_err2 = GitError::MergeConflict {
        files: vec!["file1.rs".into(), "file2.rs".into()],
    };
    let display = format!("{git_err2}");
    assert!(display.contains("conflict"));
    assert!(display.contains("file1.rs"));
}

#[test]
fn test_context_with_anyhow_error() {
    // Test that Context wrapping an anyhow error maps to InternalError
    let anyhow_err = anyhow::anyhow!("unexpected failure");
    // First convert to MaosError::Anyhow, then wrap in context
    let maos_err: MaosError = anyhow_err.into();
    let with_context = MaosError::Context {
        message: "during initialization".into(),
        source: Box::new(maos_err) as Box<dyn std::error::Error + Send + Sync>,
    };

    // Context wrapping non-MaosError should default to InternalError for anyhow
    let code: ExitCode = (&with_context).into();
    assert_eq!(code, ExitCode::InternalError);

    let display = format!("{with_context}");
    assert!(display.contains("during initialization"));
    assert!(display.contains("unexpected failure"));
}

#[test]
fn test_context_with_std_error() {
    // Test that Context wrapping a standard error maps to GeneralError
    let io_err = std::io::Error::other("io problem");
    let with_context = MaosError::Context {
        message: "while reading file".into(),
        source: Box::new(io_err),
    };

    // Context wrapping non-MaosError std errors should default to GeneralError
    let code: ExitCode = (&with_context).into();
    assert_eq!(code, ExitCode::GeneralError);
}
