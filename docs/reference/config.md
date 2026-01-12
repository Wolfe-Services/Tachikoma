# Configuration Reference

Complete reference for all Tachikoma configuration options.

## Application Configuration

Configuration is stored in YAML format at `.tachikoma/config.yaml` by default.

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `NODE_ENV` | Environment mode | `development` |
| `RUST_LOG` | Rust logging level | `info` |
| `TACHIKOMA_CONFIG_PATH` | Config file path | `.tachikoma/config.yaml` |

## File Formats

### YAML Configuration

Configuration uses YAML format. Example:

```yaml
# .tachikoma/config.yaml
backend:
  brain_model: "claude-3-5-sonnet-20241022"
  think_tank_model: "claude-3-opus-20240229"
  api_keys:
    anthropic: "${ANTHROPIC_API_KEY}"
    openai: "${OPENAI_API_KEY}"

loop_config:
  max_iterations: 100
  redline_threshold: 150000
  stop_conditions:
    - tests_pass
    - no_progress_count: 5

policies:
  require_tests_pass_to_deploy: true
  attended_mode: true
  auto_commit: false

vcs:
  type: jj  # jj (default) or git
  auto_commit_on_checkpoint: true
```

### Environment Files

Sensitive values should use environment variable substitution:

```yaml
api_keys:
  anthropic: "${ANTHROPIC_API_KEY}"  # Reads from environment
```

Or use a `.env` file (never commit this):

```bash
# .env (add to .gitignore)
ANTHROPIC_API_KEY=sk-ant-...
OPENAI_API_KEY=sk-...
```