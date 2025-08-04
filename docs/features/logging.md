# MAOS Logging System

## Overview

MAOS provides comprehensive logging for debugging, auditing, and performance monitoring. The logging system captures all orchestration activities, security events, and performance metrics while maintaining privacy and efficiency.

## Log Structure

```
.maos/
├── logs/
│   ├── maos.log              # Main application log
│   ├── security.log          # Security events
│   ├── performance.log       # Performance metrics
│   ├── tts.log              # TTS provider events
│   └── sessions/            # Per-session logs
│       └── sess-123/
│           ├── orchestration.log
│           ├── agents.log
│           └── timeline.json
└── sessions/
    └── {session_id}/
        ├── session.json
        ├── metrics.json
        └── events.json
```

## Log Levels

MAOS uses standard log levels:

```rust
// Set via environment variable
export MAOS_LOG_LEVEL=debug

// Available levels (in order of verbosity)
trace    // Very detailed debugging information
debug    // Detailed information for debugging
info     // General informational messages (default)
warn     // Warning messages for potential issues
error    // Error messages for failures
```

## Log Types

### 1. Application Logs

General MAOS operations:
```
[2024-01-20 10:30:45.123] INFO  Starting MAOS pre-tool-use hook
[2024-01-20 10:30:45.125] DEBUG Validating tool: Task
[2024-01-20 10:30:45.127] INFO  Creating worktree for backend-engineer
[2024-01-20 10:30:45.234] DEBUG Worktree created at: worktrees/backend-sess-123
```

### 2. Security Logs

All security-related events:
```
[2024-01-20 10:31:00.456] WARN  Blocked dangerous command: rm -rf /
[2024-01-20 10:31:00.457] INFO  Security violation reported to user
[2024-01-20 10:31:15.123] WARN  Blocked .env file access attempt
[2024-01-20 10:31:15.124] DEBUG Agent: frontend-sess-123, File: .env
```

### 3. Performance Logs

Execution timing and metrics:
```
[2024-01-20 10:30:45.234] PERF  Hook execution time: 8.5ms
[2024-01-20 10:30:45.235] PERF  Breakdown: validation=1.2ms, worktree=7.0ms, response=0.3ms
[2024-01-20 10:30:45.236] INFO  Performance target: <10ms ✓
```

### 4. Session Logs

Orchestration session tracking:
```json
{
  "session_id": "sess-1754237743",
  "events": [
    {
      "timestamp": "2024-01-20T10:30:45.123Z",
      "event": "session_started",
      "orchestrator": "main"
    },
    {
      "timestamp": "2024-01-20T10:30:50.456Z",
      "event": "agent_spawned",
      "agent_type": "backend-engineer",
      "worktree": "worktrees/backend-sess-1754237743"
    }
  ]
}
```

## Configuration

### Environment Variables

```bash
# Logging level
export MAOS_LOG_LEVEL=debug      # trace, debug, info, warn, error

# Enable debug mode (verbose logging)
export MAOS_DEBUG=1

# Log file rotation
export MAOS_LOG_MAX_SIZE=10MB    # Maximum log file size
export MAOS_LOG_MAX_FILES=5      # Number of rotated files to keep

# Performance logging
export MAOS_LOG_PERF=1           # Enable performance metrics
export MAOS_LOG_SLOW_OPS=5       # Log operations slower than 5ms
```

### Configuration File

```json
{
  "maos": {
    "logging": {
      "level": "info",
      "file_rotation": {
        "max_size_mb": 10,
        "max_files": 5,
        "compress": true
      },
      "performance": {
        "enabled": true,
        "slow_threshold_ms": 10
      },
      "privacy": {
        "mask_paths": true,
        "exclude_file_contents": true
      }
    }
  }
}
```

## Log Formatting

### Structured Logging

MAOS uses structured logging for easy parsing:

```rust
// JSON format for machine processing
{"timestamp":"2024-01-20T10:30:45.123Z","level":"INFO","message":"Worktree created","agent_type":"backend-engineer","session_id":"sess-123","duration_ms":8.5}

// Human-readable format for development
[2024-01-20 10:30:45.123] INFO  [sess-123] Worktree created for backend-engineer (8.5ms)
```

### Custom Fields

Contextual information in every log entry:
- `session_id` - Current orchestration session
- `agent_id` - Specific agent identifier
- `tool_name` - Claude Code tool being used
- `duration_ms` - Operation timing
- `error_code` - For error conditions

## Performance Monitoring

### Automatic Performance Tracking

Key operations are automatically timed:
```
Hook execution: <10ms target
Git operations: <50ms target
File operations: <5ms target
TTS API calls: <500ms target
```

### Performance Dashboards

View performance trends:
```bash
# Show performance summary
maos perf-summary

# Export metrics for analysis
maos export-metrics --format=csv --output=metrics.csv
```

## Privacy & Security

### Sensitive Data Protection

MAOS automatically masks sensitive information:

```
# Original log
Accessing file: /home/user/project/.env

# Masked log
Accessing file: /home/***/project/.env
```

### Excluded Content

File contents are never logged by default:
- No source code in logs
- No command outputs
- No API keys or secrets
- Only metadata and paths

### GDPR Compliance

Configure retention policies:
```json
{
  "maos": {
    "logging": {
      "retention_days": 30,
      "auto_cleanup": true,
      "anonymize_user_paths": true
    }
  }
}
```

## Debugging

### Enable Debug Mode

```bash
# Maximum verbosity for troubleshooting
export MAOS_DEBUG=1
export MAOS_LOG_LEVEL=trace

# Run MAOS command
maos pre-tool-use

# Check debug log
tail -f .maos/logs/maos.log
```

### Common Debug Scenarios

1. **Worktree Creation Issues**
   ```
   grep "worktree" .maos/logs/maos.log
   ```

2. **Security Blocks**
   ```
   grep "WARN\|ERROR" .maos/logs/security.log
   ```

3. **Performance Problems**
   ```
   grep "PERF.*[0-9]{2,}ms" .maos/logs/performance.log
   ```

4. **Session Coordination**
   ```
   cat .maos/sessions/*/timeline.json | jq
   ```

## Log Analysis Tools

### Built-in Analysis

```bash
# Show recent errors
maos logs --level=error --recent=1h

# Track specific session
maos logs --session=sess-123 --follow

# Performance analysis
maos analyze-perf --percentile=95
```

### External Tools

MAOS logs are compatible with:
- **ELK Stack**: Elasticsearch, Logstash, Kibana
- **Splunk**: Enterprise log analysis
- **Datadog**: Cloud monitoring
- **Grafana Loki**: Log aggregation

Example Logstash configuration:
```ruby
input {
  file {
    path => "/path/to/.maos/logs/*.log"
    start_position => "beginning"
    codec => "json"
  }
}

filter {
  date {
    match => [ "timestamp", "ISO8601" ]
  }
}

output {
  elasticsearch {
    hosts => ["localhost:9200"]
    index => "maos-logs-%{+YYYY.MM.dd}"
  }
}
```

## Troubleshooting

### No Logs Generated

1. Check permissions on `.maos/logs/` directory
2. Verify MAOS is running (not just Python hooks)
3. Check disk space availability

### Huge Log Files

1. Enable log rotation
2. Reduce log level from `trace` to `info`
3. Set up automatic cleanup

### Missing Information

1. Increase log level to `debug`
2. Enable `MAOS_DEBUG=1`
3. Check specific log files (security.log, etc.)

## Best Practices

### For Users

1. **Regular cleanup**: Remove old session logs periodically
2. **Monitor performance**: Check for slow operations
3. **Review security logs**: Look for unusual patterns
4. **Use appropriate levels**: `info` for normal use, `debug` for issues

### For Contributors

1. **Use structured logging**: Include context fields
2. **Log at appropriate levels**: Don't spam `info` logs
3. **Include timing**: Measure performance-critical paths
4. **Respect privacy**: Never log file contents or secrets

## Future Enhancements

- [ ] Real-time log streaming to web dashboard
- [ ] AI-powered log analysis for anomalies
- [ ] Distributed tracing for multi-agent workflows
- [ ] Log compression and archival strategies
- [ ] Integration with OpenTelemetry
- [ ] Custom log queries and dashboards

## Related Documentation

- [Configuration](../cli/configuration.md) - Logging settings
- [Architecture](../architecture/rust-cli-architecture.md) - Logging module design
- [Security](./security.md) - Security event logging