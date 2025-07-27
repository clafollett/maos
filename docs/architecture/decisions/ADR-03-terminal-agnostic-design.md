# ADR-03: Terminal-Agnostic Design

## Status
Accepted

## Context
Traditional terminal-based orchestration tools often make assumptions about the terminal environment:
- tmux requires tmux to be installed and running
- screen requires screen to be installed
- Many tools assume specific terminal emulators or capabilities
- Some tools only work locally and break over SSH
- Container and CI/CD environments often have limited terminal support

With our PTY multiplexer approach, we have the opportunity to design a system that works in ANY terminal environment without dependencies on specific terminal multiplexers or emulators.

## Decision
MAOS will implement a **terminal-agnostic design** where the CLI operates at the PTY level, making it compatible with any terminal environment that can run a command-line application.

### Design Principles

1. **No Terminal Multiplexer Dependencies**
   - Works without tmux, screen, or any other multiplexer
   - Can optionally integrate with tmux for enhanced features
   - PTY multiplexing happens within our process, not externally

2. **Universal Terminal Compatibility**
   - Native terminals: Terminal.app, iTerm2, Windows Terminal, GNOME Terminal
   - IDE terminals: VS Code, IntelliJ, Sublime Text
   - Remote access: SSH, Mosh, Telnet
   - Containerized: Docker, Kubernetes pods
   - CI/CD: GitHub Actions, Jenkins, GitLab CI
   - Web-based: Jupyter, Google Cloud Shell, AWS CloudShell

3. **Adaptive Behavior**
   - Detect terminal capabilities at runtime
   - Gracefully degrade features when capabilities are limited
   - Provide consistent core functionality everywhere

### Implementation Approach

```rust
pub struct TerminalEnvironment {
    pub tty: bool,              // Is this a TTY?
    pub term: Option<String>,   // TERM environment variable
    pub rows: u16,              // Terminal height
    pub cols: u16,              // Terminal width
    pub color: ColorSupport,    // Color capability detection
    pub unicode: bool,          // Unicode support
}

impl TerminalEnvironment {
    pub fn detect() -> Self {
        // Runtime detection of capabilities
    }
    
    pub fn adapt_output(&self, output: &str) -> String {
        // Adapt output based on terminal capabilities
    }
}
```

### Terminal Contexts

1. **Interactive Terminal**
   ```bash
   $ maos spawn backend-engineer "implement auth"
    Agent spawned with ID: backend-7f3a
   $ maos status
   ```

2. **Non-Interactive/Pipe**
   ```bash
   $ maos status --json | jq '.agents[]'
   ```

3. **Remote SSH**
   ```bash
   ssh user@server "maos orchestrate 'build API'"
   ```

4. **Container**
   ```dockerfile
   RUN maos spawn data-scientist "analyze dataset"
   ```

5. **CI/CD Pipeline**
   ```yaml
   - run: maos orchestrate "run integration tests"
   ```

### Feature Adaptation

| Feature | Full Terminal | Limited Terminal | No Terminal |
|---------|--------------|------------------|-------------|
| Colors |  Full palette | Basic 16 colors | None |
| Progress bars |  Animated | Simple | Percentage |
| Tables |  Box drawing | ASCII | CSV |
| Status updates |  In-place | New lines | Log format |
| Unicode |  Full | ASCII art | Plain text |

## Consequences

### Positive
- **Universal Compatibility**: Works anywhere you can run a CLI command
- **Zero Dependencies**: No need to install tmux or other tools
- **Remote-Friendly**: Works perfectly over SSH without special setup
- **Container-Native**: Ideal for Docker and Kubernetes environments
- **CI/CD Ready**: Functions well in automated pipelines
- **Graceful Degradation**: Always functional, even in minimal environments

### Negative
- **Feature Limitations**: Some advanced features may not be available everywhere
- **Complexity**: Detecting and adapting to different environments adds code
- **Testing Burden**: Need to test in many different terminal environments

### Neutral
- **Different from tmux-orchestrator**: We're not tied to tmux's model
- **Output Formatting**: Need multiple output formats (interactive, JSON, plain)
- **Progressive Enhancement**: Better experience in capable terminals

## Examples

### Running in Different Environments

```bash
# macOS Terminal.app
$ maos spawn backend "implement feature"
=€ Spawning backend-engineer agent...
 Agent ready (PTY: /dev/ttys003)

# SSH from Windows
PS> ssh server "maos status"
ID          ROLE              STATUS    UPTIME
backend-a1  backend-engineer  active    5m32s

# Docker container
$ docker run maos orchestrate "analyze logs"
[2024-01-15 10:32:15] Starting orchestration...
[2024-01-15 10:32:16] Spawned: orchestrator-5d4f
[2024-01-15 10:32:17] Task assigned to data-scientist-8e2a

# GitHub Actions
- run: |
    maos spawn qa "run test suite"
    maos logs qa --follow
```

## References
- [POSIX Terminal Interface](https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/termios.h.html)
- [Terminal Capabilities Database](https://invisible-island.net/ncurses/terminfo.src.html)
- Issue #5: Cross-platform PTY approach enables terminal agnosticism