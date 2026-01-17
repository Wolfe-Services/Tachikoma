# Tachikoma v1.0.0 Release Notes

**Release Date:** 2024-01-15
**Type:** Major Release

## Highlights

Tachikoma 1.0 brings autonomous AI-powered development to your desktop with multi-model support and spec-driven workflows.

## What's New

### Multi-Model Backend Support
Choose between Claude, GPT-4, Gemini, or local Ollama models for your development tasks. Switch models on the fly to leverage different strengths for different types of problems.

### Spec Forge
Collaborative specification creation using multiple AI models to brainstorm and refine your project specs. Get diverse perspectives and ensure comprehensive coverage of requirements.

### Autonomous Loop Runner
Let Tachikoma work unattended on your specifications with intelligent checkpoint management and progress tracking. Perfect for long-running development tasks.

## Improvements

- 50% faster startup time
- Reduced memory usage during long sessions
- Better error handling and recovery
- Improved progress visualization

## Bug Fixes

- Fixed crash when opening large files (#123)
- Fixed memory leak in model switching (#124)
- Resolved race condition in spec parsing

## Breaking Changes

### Configuration File Format
**What changed:** Changed configuration file structure to support multi-model settings

**Why:** The new format provides better organization and enables model-specific configuration

**Migration:** Run `tachikoma migrate-config` to automatically convert your existing config file

```diff
- "model": "claude"
+ "models": {
+   "primary": "claude",
+   "secondary": "gpt-4"
+ }
```

## Known Issues

- Ollama models may have slower initial load times on some systems
- Windows defender may flag the installer as unknown publisher (safe to ignore)

## Upgrade Instructions

### From v0.0.0

1. Download the new version for your platform
2. Run the installer (your settings will be automatically migrated)
3. If you have custom configurations, run `tachikoma migrate-config`

### Configuration Changes

Configuration files are now stored in `~/.tachikoma/config/` instead of `~/.tachikoma/`

Model settings moved to separate `models.json` file for better organization

## Downloads

- [macOS (Intel)](https://github.com/tachikoma/tachikoma/releases/download/v1.0.0/Tachikoma-1.0.0-x64.dmg)
- [macOS (Apple Silicon)](https://github.com/tachikoma/tachikoma/releases/download/v1.0.0/Tachikoma-1.0.0-arm64.dmg)
- [Windows](https://github.com/tachikoma/tachikoma/releases/download/v1.0.0/Tachikoma-Setup-1.0.0.exe)
- [Linux (AppImage)](https://github.com/tachikoma/tachikoma/releases/download/v1.0.0/Tachikoma-1.0.0.AppImage)
- [Linux (deb)](https://github.com/tachikoma/tachikoma/releases/download/v1.0.0/tachikoma_1.0.0_amd64.deb)

## Checksums

```
SHA256 checksums:
abc123... Tachikoma-1.0.0-x64.dmg
def456... Tachikoma-1.0.0-arm64.dmg
...
```

## Thank You

Thanks to all contributors who made this release possible!

@tachikoma-dev, @ai-assistant, @community-contributor

---

[Full Changelog](https://github.com/tachikoma/tachikoma/blob/main/CHANGELOG.md)
[Documentation](https://docs.tachikoma.dev)
[Report Issues](https://github.com/tachikoma/tachikoma/issues)