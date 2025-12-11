# LucAstra Configuration Schema

## Environment Variables

### LUCASTRA_CONFIG_HOME
- **Type**: Path (string)
- **Default**: `~/.lucastra` (user home + .lucastra)
- **Description**: Root directory for all LucAstra configuration files
- **Example**: `export LUCASTRA_CONFIG_HOME=/etc/lucastra`

### RUST_LOG
- **Type**: String (log filter)
- **Default**: `info` (if observability.level is not set)
- **Description**: Overrides observability level setting
- **Example**: `export RUST_LOG=debug,lucastra=trace`

## Configuration File Format

All configuration files are JSON. The primary configuration file is located at `$LUCASTRA_CONFIG_HOME/config.json`.

## Configuration Schema

### observability
Controls logging and tracing behavior.

```json
{
  "observability": {
    "level": "info",
    "file_logging": true,
    "log_dir": "./logs",
    "max_log_size_mb": 100,
    "log_files_keep": 30,
    "console_output": false,
    "json_format": true
  }
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `level` | string | `info` | Log level: `trace`, `debug`, `info`, `warn`, `error` |
| `file_logging` | boolean | `true` | Enable file-based logging |
| `log_dir` | string | `./logs` | Directory for log files |
| `max_log_size_mb` | integer | `100` | Max file size before rotation (MB) |
| `log_files_keep` | integer | `30` | Number of rotated logs to retain |
| `console_output` | boolean | `false` | Print logs to stdout |
| `json_format` | boolean | `true` | Use JSON format (false = human-readable) |

### metrics
Controls metrics collection and export.

```json
{
  "metrics": {
    "enabled": true,
    "export_to_file": true,
    "export_dir": "./metrics",
    "export_interval_secs": 3600
  }
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `true` | Enable metrics collection |
| `export_to_file` | boolean | `true` | Export metrics to JSON file |
| `export_dir` | string | `./metrics` | Directory for metrics exports |
| `export_interval_secs` | integer | `3600` | Export interval in seconds |

### security
Controls file access and sandboxing.

```json
{
  "security": {
    "allowed_host_dirs": [
      "~/Documents",
      "~/Downloads"
    ],
    "allow_host_read": true,
    "allow_host_write": false,
    "allow_usb": false
  }
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `allowed_host_dirs` | string[] | `[]` | Whitelisted directories (~ expands to $HOME) |
| `allow_host_read` | boolean | `true` | Allow reading from host filesystem |
| `allow_host_write` | boolean | `false` | Allow writing to host filesystem |
| `allow_usb` | boolean | `false` | Allow USB device access |

## Complete Configuration Example

```json
{
  "observability": {
    "level": "debug",
    "file_logging": true,
    "log_dir": "./logs",
    "max_log_size_mb": 50,
    "log_files_keep": 10,
    "console_output": true,
    "json_format": false
  },
  "metrics": {
    "enabled": true,
    "export_to_file": true,
    "export_dir": "./metrics",
    "export_interval_secs": 30
  },
  "security": {
    "allowed_host_dirs": [
      "~/Documents",
      "~/Downloads",
      "~/Projects"
    ],
    "allow_host_read": true,
    "allow_host_write": true,
    "allow_usb": false
  }
}
```

## Configuration Directory Structure

```
$LUCASTRA_CONFIG_HOME/
├── config.json              # Main configuration (required)
├── logs/                    # Log files (created automatically)
│   └── lucastra.log
│   └── lucastra-2025-12-10.log
├── metrics/                 # Metrics exports (created automatically)
│   └── metrics-2025-12-10T15-30-00.json
└── audit/                   # File access audit logs (created automatically)
    └── audit.jsonl
```

## Log Levels

| Level | Usage |
|-------|-------|
| `trace` | Very detailed diagnostic info (function entry/exit, variable values) |
| `debug` | Detailed debugging info (config loading, state changes) |
| `info` | General informational messages (startup, shutdown, key events) |
| `warn` | Warning messages (deprecated features, recoverable errors) |
| `error` | Error messages (failures, exceptions) |

## Metrics Export Format

Metrics are exported as JSON with the following structure:

```json
{
  "timestamp": "2025-12-10T15:30:00Z",
  "total_commands": 42,
  "total_tools_executed": 156,
  "tool_successes": 150,
  "tool_failures": 6,
  "tool_success_rate": 96.15,
  "total_search_queries": 23,
  "total_search_latency_ms": 1234
}
```

## Audit Log Format

File access audit logs are stored in JSON Lines format (one JSON object per line):

```json
{"timestamp":"2025-12-10T15:30:00Z","operation":"read","path":"/home/user/Documents/file.txt","status":"success","user":"user"}
{"timestamp":"2025-12-10T15:30:01Z","operation":"write","path":"/home/user/Documents/output.txt","status":"denied","reason":"not_in_whitelist","user":"user"}
```

## Best Practices

### Development Environment
- Use `level: "debug"` for detailed logging
- Enable `json_format: false` for human-readable output
- Set `export_interval_secs: 30` for frequent metrics export
- Use `file_logging: true` and `console_output: true` for dual output

### Production Environment
- Use `level: "info"` or `level: "warn"`
- Enable `json_format: true` for structured logging
- Set `export_interval_secs: 3600` (once per hour)
- Use `file_logging: true` only; disable `console_output`
- Use smaller `max_log_size_mb` (50-100 MB) with `log_files_keep: 30`

### Security-Sensitive Environments
- Whitelist specific directories only (empty list default)
- Use `allow_host_write: false`
- Use `allow_usb: false`
- Monitor audit logs regularly
- Use `level: "info"` for compliance logging

## Troubleshooting

### Logs not appearing
1. Check `LUCASTRA_CONFIG_HOME` is set: `echo $LUCASTRA_CONFIG_HOME`
2. Verify `config.json` exists at that path
3. Check file permissions on log directory
4. Try `console_output: true` to see immediate output

### Metrics not exporting
1. Ensure `metrics.enabled: true` in config
2. Check `metrics.export_dir` is writable
3. Verify `metrics.export_interval_secs` is not 0
4. Check logs for export errors at `debug` level

### High disk usage from logs
1. Reduce `max_log_size_mb`
2. Reduce `log_files_keep`
3. Lower log `level` to reduce noise
4. Disable `console_output` (stdout can buffer logs)

---

**Configuration Version**: 1.0.0  
**Last Updated**: 2025-12-10
