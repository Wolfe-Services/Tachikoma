# Tachikoma Architecture

Tachikoma is a multi-process agentic coding platform using the Ralph Wiggum Loop pattern.

## MVP Boundary

**MVP is desktop-local only.** Server/database/auth phases (15-20) are post-MVP SaaS features.

| MVP Scope | Deferred to Post-MVP |
|-----------|---------------------|
| Electron + Rust (NAPI) | Axum server |
| Local filesystem | PostgreSQL |
| SQLite for indexing | Multi-tenant auth |
| Single user | Feature flags |
| jj VCS | Analytics/audit SaaS |

## Core Decisions

### State Management

```
Filesystem = Source of Truth
├── specs/*.md          (canonical spec content)
├── .tachikoma/config.yaml  (configuration)
├── mission logs        (execution history)
└── working files       (code being edited)

SQLite = Index/Cache Only
├── spec search index
├── mission history queries
├── analytics aggregations
└── NOT authoritative - can be rebuilt from FS
```

### Version Control: jj-first

**jj (Jujutsu) is the primary VCS**, with git compatibility for remotes.

Why jj for agentic coding:
- **Concurrent edits**: First-class support, automatic rebasing
- **Conflict handling**: Can commit with conflicts, resolve later
- **Undo/redo**: Operation log makes any action reversible
- **Working copy**: Not special - just another commit
- **No staging area**: Simpler mental model for agents

Git compatibility:
- Push/pull to git remotes (GitHub, GitLab)
- Colocated mode: jj + git in same repo
- CI/CD integration via git export

### Configuration

- Format: **YAML** (not TOML)
- Location: `.tachikoma/config.yaml`
- Secrets: Environment variable substitution

## Core Components

### Rust Backend (`crates/`)

| Crate | Purpose |
|-------|---------|
| `tachikoma-common-core` | Shared types, IDs, timestamps |
| `tachikoma-common-config` | Configuration types and loading |
| `tachikoma-primitives` | Five primitives (read, list, bash, edit, search) |
| `tachikoma-backends` | LLM backend abstraction |
| `tachikoma-loop` | Ralph Loop runner |
| `tachikoma-vcs` | jj-first VCS integration |
| `tachikoma-forge-types` | Spec Forge multi-model types |

### Electron Shell (`electron/`)

- Main process coordination
- Security sandboxing (CSP, context isolation)
- Native system integration
- IPC bridge to Rust via NAPI

### SvelteKit Frontend (`web/`)

- Glassmorphic design system
- Modern web UI with TypeScript
- Component-based architecture
- Real-time mission monitoring

### IPC Communication

- Type-safe JSON messages
- Callback wrapper tracking (prevents memory leaks)
- Channel allowlists for security
- Async/await patterns

## The Ralph Wiggum Loop

```
┌─────────────────────────────────────────────────────┐
│                  RALPH LOOP                          │
├─────────────────────────────────────────────────────┤
│  1. Load spec/prompt                                │
│  2. Send to LLM with tools                          │
│  3. Execute tool calls (primitives)                 │
│  4. Check stop conditions                           │
│  5. If not stopped: goto 2                          │
│  6. If redline: reboot with fresh context           │
└─────────────────────────────────────────────────────┘
```

### Loop Modes

| Mode | Description |
|------|-------------|
| **Attended** | Human approves each checkpoint |
| **Unattended** | Runs autonomously with stop conditions |

### Stop Conditions

- Tests pass
- Max iterations reached
- No progress detected (N iterations)
- Redline threshold exceeded
- Cost limit reached

## Security Model

Primitives enforce security at the execution layer:

| Constraint | Enforcement |
|------------|-------------|
| Path allowlist | `SecurityPolicy.can_read/write()` |
| Command blocklist | Dangerous commands rejected |
| Workspace boundary | All paths must be within workspace |
| Secret redaction | Output sanitized before logging |
| Audit logging | All executions recorded |

## Process Boundaries

### Desktop Mode (MVP)

```
┌─────────────────────────────────────────┐
│            Electron Main                 │
│  ┌─────────────────────────────────┐    │
│  │      Renderer (SvelteKit)       │    │
│  └──────────────┬──────────────────┘    │
│                 │ IPC                    │
│  ┌──────────────▼──────────────────┐    │
│  │    Rust Backend (NAPI)          │    │
│  │  ┌──────────┐ ┌──────────┐      │    │
│  │  │Primitives│ │   Loop   │      │    │
│  │  └──────────┘ └──────────┘      │    │
│  │  ┌──────────┐ ┌──────────┐      │    │
│  │  │ Backends │ │   VCS    │      │    │
│  │  └──────────┘ └──────────┘      │    │
│  └─────────────────────────────────┘    │
└─────────────────────────────────────────┘
```

### Server Mode (Post-MVP)

```
┌───────────────┐     ┌───────────────────┐
│  Desktop App  │────▶│   Axum Server     │
└───────────────┘     │  ┌─────────────┐  │
                      │  │   Auth      │  │
┌───────────────┐     │  ├─────────────┤  │
│   Web Client  │────▶│  │   Loop      │  │
└───────────────┘     │  ├─────────────┤  │
                      │  │   DB/Cache  │  │
                      │  └─────────────┘  │
                      └───────────────────┘
```

## Project Structure

```
tachikoma/
├── crates/              # Rust backend crates
│   ├── tachikoma-common-core/
│   ├── tachikoma-common-config/
│   ├── tachikoma-primitives/
│   ├── tachikoma-backends/
│   ├── tachikoma-loop/
│   ├── tachikoma-vcs/
│   └── tachikoma-forge-types/
├── electron/            # Electron main process
│   ├── main/
│   └── preload/
├── web/                 # SvelteKit frontend
│   ├── src/
│   │   ├── lib/
│   │   │   ├── components/
│   │   │   ├── stores/
│   │   │   └── styles/
│   │   └── routes/
│   └── static/
├── specs/               # Specifications (THE PIN)
│   ├── README.md        # Spec index
│   └── phase-XX-*/      # Phase directories
└── docs/                # Documentation
    ├── architecture.md  # This file
    └── reference/       # Reference docs
```

## Technology Stack

| Component | Technology |
|-----------|------------|
| Language | Rust (backend), TypeScript (frontend) |
| Framework | SvelteKit (web), Electron (desktop) |
| Database | SQLite (MVP), PostgreSQL (SaaS) |
| VCS | **jj** (primary), git (compatibility) |
| Styling | Glassmorphic CSS, design tokens |
| Testing | Vitest, Cargo test |
