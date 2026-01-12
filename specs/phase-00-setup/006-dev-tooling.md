# 006 - Development Tooling

**Phase:** 0 - Setup
**Spec ID:** 006
**Status:** Planned
**Dependencies:** 003-electron-shell, 004-svelte-integration
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Configure hot module replacement, development scripts, and debugging tools for efficient development workflow.

---

## Acceptance Criteria

- [x] HMR working for Svelte components
- [x] Electron main process restarts on changes
- [x] Concurrent dev server script
- [x] Source maps configured
- [x] Chrome DevTools accessible
- [x] Debug configurations for VS Code

---

## Implementation Details

### 1. Root Development Script

Update root `package.json`:

```json
{
  "scripts": {
    "dev": "concurrently -k -n web,electron \"npm run dev:web\" \"npm run dev:electron\"",
    "dev:web": "cd web && npm run dev",
    "dev:electron": "wait-on http://localhost:5173 && cd electron && npm run dev",
    "dev:rust": "cargo watch -x check -x test",
    "build": "npm run build:web && npm run build:electron && npm run build:rust",
    "build:web": "cd web && npm run build",
    "build:electron": "cd electron && npm run build",
    "build:rust": "cargo build --release"
  },
  "devDependencies": {
    "concurrently": "^8.2.0",
    "wait-on": "^7.2.0"
  }
}
```

### 2. Electron Dev Configuration

Update `electron/package.json` scripts:

```json
{
  "scripts": {
    "dev": "electron-vite dev",
    "dev:main": "electron-vite dev --mode main",
    "build": "electron-vite build",
    "preview": "electron-vite preview"
  }
}
```

### 3. VS Code Launch Configuration

Create `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Debug Electron Main",
      "type": "node",
      "request": "launch",
      "cwd": "${workspaceFolder}/electron",
      "runtimeExecutable": "${workspaceFolder}/electron/node_modules/.bin/electron",
      "runtimeArgs": [
        "--remote-debugging-port=9222",
        "."
      ],
      "env": {
        "ELECTRON_RENDERER_URL": "http://localhost:5173"
      },
      "sourceMaps": true
    },
    {
      "name": "Debug Electron Renderer",
      "type": "chrome",
      "request": "attach",
      "port": 9222,
      "webRoot": "${workspaceFolder}/web"
    },
    {
      "name": "Debug Rust Tests",
      "type": "lldb",
      "request": "launch",
      "cargo": {
        "args": ["test", "--no-run", "--lib"],
        "filter": {
          "kind": "lib"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}"
    }
  ],
  "compounds": [
    {
      "name": "Debug Full App",
      "configurations": ["Debug Electron Main", "Debug Electron Renderer"]
    }
  ]
}
```

### 4. VS Code Tasks

Create `.vscode/tasks.json`:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "dev",
      "type": "shell",
      "command": "npm run dev",
      "isBackground": true,
      "problemMatcher": {
        "owner": "typescript",
        "pattern": {
          "regexp": "^(.*):(\\d+):(\\d+):\\s+(error|warning):\\s+(.*)$",
          "file": 1,
          "line": 2,
          "column": 3,
          "severity": 4,
          "message": 5
        },
        "background": {
          "activeOnStart": true,
          "beginsPattern": "^.*Starting.*$",
          "endsPattern": "^.*ready.*$"
        }
      }
    },
    {
      "label": "build",
      "type": "shell",
      "command": "npm run build",
      "group": {
        "kind": "build",
        "isDefault": true
      }
    },
    {
      "label": "cargo check",
      "type": "shell",
      "command": "cargo check --workspace",
      "problemMatcher": "$rustc"
    },
    {
      "label": "cargo test",
      "type": "shell",
      "command": "cargo test --workspace",
      "group": {
        "kind": "test",
        "isDefault": true
      }
    }
  ]
}
```

### 5. VS Code Settings

Create `.vscode/settings.json`:

```json
{
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "[svelte]": {
    "editor.defaultFormatter": "svelte.svelte-vscode"
  },
  "rust-analyzer.checkOnSave.command": "clippy",
  "typescript.tsdk": "node_modules/typescript/lib",
  "svelte.enable-ts-plugin": true
}
```

### 6. Recommended Extensions

Create `.vscode/extensions.json`:

```json
{
  "recommendations": [
    "svelte.svelte-vscode",
    "rust-lang.rust-analyzer",
    "dbaeumer.vscode-eslint",
    "esbenp.prettier-vscode",
    "bradlc.vscode-tailwindcss",
    "vadimcn.vscode-lldb"
  ]
}
```

---

## Testing Requirements

1. `npm run dev` starts all services
2. Changes to Svelte files hot reload
3. Changes to Electron main restart app
4. VS Code debugging attaches correctly

---

## Related Specs

- Depends on: [003-electron-shell.md](003-electron-shell.md), [004-svelte-integration.md](004-svelte-integration.md)
- Next: [007-build-system.md](007-build-system.md)
