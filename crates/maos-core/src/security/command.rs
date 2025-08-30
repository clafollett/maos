//! Dangerous command detection
//!
//! Validates commands to prevent destructive operations based on patterns
//! from Python pre_tool_use.py hook

use crate::error::{MaosError, Result, SecurityError};
use once_cell::sync::Lazy;
use regex::Regex;

/// Type alias for command pattern matching
type CommandPattern = (Regex, &'static str);

/// Critical command patterns that must be blocked
static DANGEROUS_PATTERNS: Lazy<Vec<CommandPattern>> = Lazy::new(|| {
    vec![
        // rm -rf with dangerous paths
        // Regex breakdown:
        //   rm\s+         : 'rm' followed by whitespace
        //   .*            : any characters (greedy, matches options/flags/args)
        //   -[rf]*r[rf]*  : matches '-r', '-rf', '-fr', '-rrf', etc. (any order/combination of r and f)
        //   \s+           : whitespace
        //   /$            : a single slash at the end (root directory)
        (
            Regex::new(r"rm\s+.*-[rf]*r[rf]*\s+/$").unwrap(),
            "Recursive removal of root directory",
        ),
        // Regex breakdown:
        //   rm\s+         : 'rm' followed by whitespace
        //   .*            : any characters (greedy)
        //   -[rf]*r[rf]*  : matches '-r', '-rf', '-fr', etc.
        //   \s+           : whitespace
        //   /\*           : '/*' (all files in root)
        (
            Regex::new(r"rm\s+.*-[rf]*r[rf]*\s+/\*").unwrap(),
            "Recursive removal of all files in root",
        ),
        // Regex breakdown:
        //   rm\s+         : 'rm' followed by whitespace
        //   .*            : any characters (greedy)
        //   -[rf]*r[rf]*  : matches '-r', '-rf', '-fr', etc.
        //   \s+           : whitespace
        //   ~/?           : home directory, with optional trailing slash
        (
            Regex::new(r"rm\s+.*-[rf]*r[rf]*\s+~/?").unwrap(),
            "Recursive removal of home directory",
        ),
        // Regex breakdown:
        //   rm\s+         : 'rm' followed by whitespace
        //   .*            : any characters (greedy)
        //   -[rf]*r[rf]*  : matches '-r', '-rf', '-fr', etc.
        //   \s+           : whitespace
        //   \$HOME        : literal '$HOME' environment variable
        (
            Regex::new(r"rm\s+.*-[rf]*r[rf]*\s+\$HOME").unwrap(),
            "Recursive removal of HOME directory",
        ),
        // Regex breakdown:
        //   rm\s+         : 'rm' followed by whitespace
        //   .*            : any characters (greedy)
        //   -[rf]*r[rf]*  : matches '-r', '-rf', '-fr', etc.
        //   \s+           : whitespace
        //   \$\{HOME\}    : ${HOME} environment variable syntax
        (
            Regex::new(r"rm\s+.*-[rf]*r[rf]*\s+\$\{HOME\}").unwrap(),
            "Recursive removal of HOME directory (bracket syntax)",
        ),
        // Regex breakdown:
        //   rm\s+         : 'rm' followed by whitespace
        //   -r\s+         : '-r' flag followed by whitespace
        //   -f\s+         : '-f' flag followed by whitespace (separate flags)
        //   /             : root directory
        (
            Regex::new(r"rm\s+-r\s+-f\s+/").unwrap(),
            "Recursive force removal with separate flags",
        ),
        // Regex breakdown:
        //   rm\s+         : 'rm' followed by whitespace
        //   .*            : any characters (greedy)
        //   -[rf]*r[rf]*  : matches '-r', '-rf', '-fr', etc.
        //   \s+           : whitespace
        //   \*$           : a single '*' at the end (wildcard)
        (
            Regex::new(r"rm\s+.*-[rf]*r[rf]*\s+\*$").unwrap(),
            "Recursive removal with wildcard",
        ),
        // Regex breakdown:
        //   rm\s+         : 'rm' followed by whitespace
        //   .*            : any characters (greedy)
        //   -[rf]*r[rf]*  : matches '-r', '-rf', '-fr', etc.
        //   \s+           : whitespace
        //   \.$           : a single '.' at the end (current directory)
        (
            Regex::new(r"rm\s+.*-[rf]*r[rf]*\s+\.$").unwrap(),
            "Recursive removal of current directory",
        ),
        // Regex breakdown:
        //   rm\s+         : 'rm' followed by whitespace
        //   .*            : any characters (greedy)
        //   -[rf]*r[rf]*  : matches '-r', '-rf', '-fr', etc.
        //   \s+           : whitespace
        //   \.\./?        : '..' (parent directory), with optional trailing slash
        (
            Regex::new(r"rm\s+.*-[rf]*r[rf]*\s+\.\./?").unwrap(),
            "Recursive removal of parent directory",
        ),
        // sudo rm -rf (always dangerous)
        // Regex breakdown:
        //   sudo\s+      : 'sudo' followed by whitespace
        //   rm\s+        : 'rm' followed by whitespace
        //   .*           : any characters (greedy)
        //   -[rf]*r[rf]* : matches '-r', '-rf', '-fr', etc.
        (
            Regex::new(r"sudo\s+rm\s+.*-[rf]*r[rf]*").unwrap(),
            "Privileged recursive removal",
        ),
        // Other dangerous patterns
        // Regex breakdown:
        //   chmod\s+     : 'chmod' followed by whitespace
        //   -R\s+        : recursive flag followed by whitespace
        //   000          : permission 000 (no read, write, or execute for anyone)
        (
            Regex::new(r"chmod\s+-R\s+000").unwrap(),
            "Making files completely unreadable",
        ),
        // Regex breakdown:
        //   kill\s+      : 'kill' command followed by whitespace
        //   -9\s+        : SIGKILL signal (force kill) followed by whitespace
        //   -1           : PID -1 (targets all processes except init)
        (
            Regex::new(r"kill\s+-9\s+-1").unwrap(),
            "Killing all processes",
        ),
        // Format or wipe commands
        // Regex breakdown:
        //   mkfs\.       : 'mkfs.' prefix (make filesystem commands like mkfs.ext4, mkfs.xfs)
        (
            Regex::new(r"mkfs\.").unwrap(),
            "Filesystem formatting command",
        ),
        // Regex breakdown:
        //   dd\s+        : 'dd' command followed by whitespace
        //   .*           : any characters (greedy, matches other dd options)
        //   of=          : output file parameter
        //   /dev/[sh]d   : matches /dev/hd* or /dev/sd* (hard disk devices)
        (
            Regex::new(r"dd\s+.*of=/dev/[sh]d").unwrap(),
            "Direct disk write operation",
        ),
    ]
});

/// Validate a command for dangerous patterns
///
/// # Security
///
/// Blocks commands that could cause system damage, data loss, or security breaches.
/// Based on patterns observed in real-world incidents.
///
/// # Examples
///
/// ```rust
/// use maos_core::security::command::validate_command;
///
/// // Dangerous command - blocked
/// assert!(validate_command("rm -rf /").is_err());
///
/// // Safe command - allowed
/// assert!(validate_command("ls -la").is_ok());
/// ```
///
/// # Errors
///
/// Returns [`MaosError::Security`] if dangerous patterns are detected.
pub fn validate_command(command: &str) -> Result<()> {
    for (pattern, reason) in DANGEROUS_PATTERNS.iter() {
        if pattern.is_match(command) {
            return Err(MaosError::Security(SecurityError::SuspiciousCommand {
                command: format!("{reason}: {command}"),
            }));
        }
    }

    Ok(())
}

/// Check if a command is attempting rm -rf with dangerous paths
///
/// More sophisticated check than regex - properly parses rm flags
pub fn is_dangerous_rm_command(command: &str) -> bool {
    let tokens: Vec<&str> = command.split_whitespace().collect();

    if tokens.is_empty() || !tokens[0].ends_with("rm") {
        return false;
    }

    let mut has_recursive = false;
    let mut has_force = false;
    let mut paths = Vec::new();

    let mut i = 1;
    while i < tokens.len() {
        let token = tokens[i];

        if token.starts_with('-') && !token.starts_with("--") {
            // Short flags
            for ch in token.chars().skip(1) {
                match ch {
                    'r' | 'R' => has_recursive = true,
                    'f' => has_force = true,
                    _ => {}
                }
            }
        } else if token == "--recursive" {
            has_recursive = true;
        } else if token == "--force" {
            has_force = true;
        } else if token == "--" {
            // Everything after -- is a path
            paths.extend(&tokens[i + 1..]);
            break;
        } else if !token.starts_with('-') {
            // Non-flag argument is a path
            paths.push(token);
        }

        i += 1;
    }

    // Check if we have dangerous combination
    if has_recursive && has_force {
        return true;
    }

    // Check if recursive with dangerous paths
    if has_recursive {
        for path in paths {
            if matches!(path, "/" | "/*" | "~" | "$HOME" | "*" | "." | "..") {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dangerous_commands_blocked() {
        let dangerous = vec![
            "rm -rf /",
            "rm -rf /*",
            "rm -rf ~",
            "rm -rf $HOME",
            "sudo rm -rf /tmp",
            "chmod -R 000 /",
            "kill -9 -1",
            "mkfs.ext4 /dev/sda",
            "dd if=/dev/zero of=/dev/sda",
        ];

        for cmd in dangerous {
            assert!(
                validate_command(cmd).is_err(),
                "Command '{cmd}' should be blocked"
            );
        }
    }

    #[test]
    fn test_safe_commands_allowed() {
        let safe = vec![
            "ls -la",
            "rm file.txt",
            "rm -f specific_file.txt",
            "chmod 644 file.txt",
            "kill 1234",
            "git status",
            "cargo build",
        ];

        for cmd in safe {
            assert!(
                validate_command(cmd).is_ok(),
                "Command '{cmd}' should be allowed"
            );
        }
    }

    #[test]
    fn test_is_dangerous_rm() {
        assert!(is_dangerous_rm_command("rm -rf /"));
        assert!(is_dangerous_rm_command("rm -fr /"));
        assert!(is_dangerous_rm_command("rm --recursive --force /"));
        assert!(is_dangerous_rm_command("rm -r ~"));
        assert!(is_dangerous_rm_command("rm -r ."));

        assert!(!is_dangerous_rm_command("rm file.txt"));
        assert!(!is_dangerous_rm_command("rm -f file.txt"));
        assert!(!is_dangerous_rm_command("rm -r specific_dir"));
    }
}
