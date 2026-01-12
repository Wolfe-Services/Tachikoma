# Tachikoma — Agentic Coding Platform Specification
## Derived from Geoffrey Huntley's "The Ralph Wiggum Loop from 1st Principles"

**Product Name:** Tachikoma  
**Tagline:** *"Your squad of tireless AI coders"*  
**Version:** 2.0  
**Date:** 2026-01-11  
**Source:** Video analysis + transcript + OCR extraction + distilled operational tips

> **North Star:** "Cursor for agentic application work" with a **Ghost in the Shell Tachikoma vibe** — memable, friendly, and usable by non-technical people.

---

## Table of Contents

0. [Product Positioning & Tachikoma Vibe](#0-product-positioning--tachikoma-vibe)
1. [Executive Summary](#1-executive-summary)
2. [Core Principles](#2-core-principles)
3. [Architecture Overview](#3-architecture-overview)
4. [The Ralph Loop Pattern](#4-the-ralph-loop-pattern)
5. [Specification System](#5-specification-system)
6. [Spec Forge: Multi-Model Brainstorming](#6-spec-forge-multi-model-brainstorming)
7. [File System as State / Backpressure](#7-file-system-as-state--backpressure)
8. [Tachikoma Architecture (Autonomous Agents)](#8-tachikoma-architecture-autonomous-agents)
9. [Implementation Plan](#9-implementation-plan)
10. [Appendix: Tachikoma Reference Architecture](#10-appendix-tachikoma-reference-architecture)

---

## 0. Product Positioning & Tachikoma Vibe

### 0.1 What is Tachikoma?

Tachikoma is a **developer (and non-developer) friendly agentic workbench** that turns "big intentions" into **small verified changes**, repeatedly, with the file system as the source of truth.

It should feel like:
- **Cursor**: point it at a repo, ask for work, watch diffs/tests happen
- **A Tachikoma squadmate** from Ghost in the Shell: curious, chatty, occasionally derpy, but relentlessly helpful and self-correcting

### 0.2 Ghost in the Shell Aesthetic (Memable Hooks)

| Concept | Tachikoma Name | Description |
|---------|----------------|-------------|
| A task / unit of work | **Mission** | "Deploy Tachikoma on this mission" |
| The human operator | **Major** | You're in command; Tachikomas report to you |
| Fast agentic model | **Brain** | Action-biased, tool-calling squirrel energy |
| Deep reasoning model | **Think Tank** | Oracle for planning, review, debugging |
| Autonomous agent pod | **Tachikoma** | The little blue spider-tank doing the work |
| Spec brainstorm session | **Spec Forge** | Models argue and refine specs together |
| Fresh context restart | **Reboot** | "Context is cooked — initiating reboot" |

**Memable one-liners for UI/CLI:**
- *"Tachikoma on the case!"*
- *"Mission complete. Tests green. Deploying."*
- *"Context redlining — requesting reboot, Major."*
- *"Think Tank says: 'This approach has issues…'"*
- *"Spec Forge: Round 3 — convergence detected."*

### 0.3 Who It's For (Personas)

| Persona | Needs |
|---------|-------|
| **Non-technical operator ("Major")** | Plain-language outcomes ("add a form", "fix the bug") with safe guardrails and clear progress |
| **Technical builder** | Deterministic, scriptable harness that doesn't spiral; full control when needed |
| **Team lead** | Scale throughput with high oversight, low control; audit trail for everything |

### 0.4 North-Star UX Requirements

- **One mission at a time**: UI/CLI visually enforces "one context = one task"
- **Show the receipts**: every step has an artifact (diff, test run, plan checkbox, log)
- **Mode toggles**:
  - **Attended** ("In the room with Tachikoma"): pause/approve checkpoints
  - **Unattended** ("Night shift"): runs guarded by policies and hard gates
- **Plain language by default**: "What changed?" "Why?" "What's next?" "Is it safe?"
- **Jargon behind 'Details'**: non-technical users see outcomes, technical users can drill down

### 0.5 CLI Requirements (Scriptable, Boring, Reliable)

The CLI is the "engine room":
- **Non-interactive friendly** (flags for CI/CD)
- **Deterministic outputs** (machine-readable JSON option)
- **Safe defaults** (dry-run, gated deploy)

**Commands:**
```bash
tachikoma init              # Scaffold specs/, prompt.md, templates
tachikoma run               # Single mission (attended by default)
tachikoma loop              # Continuous missions with backoff + stop conditions
tachikoma forge             # Spec Forge: multi-model brainstorming session
tachikoma doctor            # Diagnostics: tool availability, limits, env sanity
tachikoma tools             # List/verify the "five primitives"
tachikoma backends          # List available model backends
tachikoma config            # View/edit .tachikoma/config.yaml
```

### 0.6 Model-Agnostic Backend

Tachikoma is **not locked to Claude**. The harness supports swappable backends:

| Backend | CLI Example | Notes |
|---------|-------------|-------|
| Claude Code | `claude --dangerously-skip-permissions` | Current baseline |
| OpenAI Codex CLI | `codex` | When available |
| Gemini CLI | `gemini-cli` | Google's offering |
| Local (Ollama) | `ollama run codellama` | Air-gapped / cost-sensitive |
| Custom | User-defined wrapper | Any tool-calling LLM |

**Configuration (.tachikoma/config.yaml):**
```yaml
backend:
  brain: claude              # Fast agentic model (default)
  think_tank: o3             # Deep reasoning oracle
  forge_participants:        # For Spec Forge sessions
    - claude
    - gemini
    - codex

loop:
  max_iterations: 100
  stop_on:
    - redline
    - test_fail_streak:3
    - no_progress:5

policies:
  deploy_requires_tests: true
  attended_by_default: true
```

### 0.7 The Five Core Primitives (Minimum Viable Toolbelt)

Every agent backend MUST provide these five tools. From field experience: **more tools = worse outcomes**.

| Primitive | Purpose | Implementation |
|-----------|---------|----------------|
| **read_file** | Read file contents | `os.ReadFile` / `fs.readFileSync` |
| **list_files** | List directory contents | `filepath.Walk` / `fs.readdirSync` |
| **bash** | Execute shell commands | `exec.Command("bash", "-c", cmd)` |
| **edit_file** | Modify files (unique match required) | Search-replace with uniqueness check |
| **code_search** | Search codebase patterns | `ripgrep` (`rg --json`) |

> "What if I told you there is no magic for indexing source code? Nearly every coding tool uses the open source ripgrep binary under the hood."

**Anti-pattern: Tool sprawl**
- Cursor caps MCP tools at 40 for good reason
- Easy to allocate 76k tokens just for tool definitions, leaving only 100k usable
- **Cardinal rule:** The more you allocate to context, the worse performance gets

### 0.8 Non-Goals (Explicitly)

- **"Secure-by-prompt" claims**: Security comes from mechanisms (sandbox, policy, audit), not vibes
- **Infinite tool sprawl**: Fewer tools = better outcomes (context is precious)
- **Replacing human judgment**: Tachikoma augments; Major decides

---

## 1. Executive Summary

The Ralph Wiggum Loop is an agentic coding pattern that enables **autonomous 24/7 software development** at approximately **$10.42 USD/hour** using Claude Code (or similar LLM coding assistants).

### Key Innovation
Instead of fighting context window limitations with compaction (lossy), the Ralph Loop uses:
- **Deterministic memory allocation** ("malloc") of the context window
- **Fresh sessions per task** (avoid context rot)
- **File system as state** (specs, implementation plans as the source of truth)
- **High oversight, low control** (let LLM decide what to do, human steers)

### Economics
```
API Cost (Sonnet 4.5): ~$250/day running continuously
Token Budget: ~$100–$500/day per developer (varies by model/usage)
Output: Multiple days/weeks of human developer work per hour
Effective Rate: $10.42/hour
ROI: Astronomical for well-specified projects
```

---

## 2. Core Principles

### 2.1 First Principles Thinking
> "Don't start with a jackhammer like Ralph. Learn how to use a screwdriver first."

1. **Question Everything**: "Was this designed for humans?" → If yes, can we cut it?
2. **Minimize Context Usage**: Less context = less sliding = better outcomes
3. **Avoid Compaction**: Compaction is lossy → Loss of "pin" → Bad results

### 2.2 One Context = One Task (The #1 Rule)
> "My #1 recommendation is to use a context window for one task, and one task only. If your coding agent is misbehaving, it's time to create a new context window. If the bowling ball is in the gutter, there's no saving it."

- One mission per context
- If stuck, **reboot** (fresh context) rather than "reason harder"
- Once data is `malloc()`'ed into context, it cannot be `free()`'d unless you start fresh

### 2.3 Don't Redline the Context Window
> "Red is bad because it results in audio clipping and muddy mixes. The same applies to LLMs."

- Claude 3.7's advertised: **200k tokens**
- Quality clips at: **147k-152k tokens**
- After system prompts + harness allocation: ~**176k usable**

When redlining:
- Tool call invocations start failing
- The LLM "forgets" how to use registered tools
- Quality degrades dramatically

### 2.4 Models Have Roles (Not Interchangeable)

| Role | Characteristics | Use Case |
|------|-----------------|----------|
| **Brain (Agentic)** | Tool-calling, action-biased ("digital squirrel") | Implementation, execution |
| **Think Tank (Oracle)** | High reasoning, deep thinking | Planning, review, debugging |

> "A model is either an oracle or agentic. It's never both."

### 2.5 The Rift
Software development is now bifurcated:
- **Those who get it**: Use agentic coding, output weeks of work daily
- **Those who don't**: Still doing everything manually

### 2.6 Software Development vs Software Engineering
- **Software Development**: Now automated with trivial bash loops (~$10/hr)
- **Software Engineering**: Designing the backpressure systems, keeping the locomotive on track

---

## 3. Architecture Overview

### 3.1 System Components

```
┌─────────────────────────────────────────────────────────────────────┐
│                       TACHIKOMA PLATFORM                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐            │
│  │   Specs     │───▶│   Prompt    │───▶│   Brain     │            │
│  │  (Pin)      │    │  (Mission)  │    │ (Agentic)   │            │
│  └─────────────┘    └─────────────┘    └──────┬──────┘            │
│        ▲                                       │                   │
│        │                                       ▼                   │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐            │
│  │   README    │◀───│   Impl      │◀───│   Code      │            │
│  │  (Lookup)   │    │   Plan      │    │   Changes   │            │
│  └─────────────┘    └─────────────┘    └─────────────┘            │
│                                               │                    │
│        ┌──────────────────────────────────────┤                    │
│        │                                      ▼                    │
│  ┌─────────────┐                       ┌─────────────┐            │
│  │ Think Tank  │◀──────(tool call)─────│   Tests     │            │
│  │  (Oracle)   │                       │   + Deploy  │            │
│  └─────────────┘                       └─────────────┘            │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.2 Key Directories
```
project/
├── specs/                      # Specification files (THE PIN)
│   ├── README.md               # Lookup table with descriptors
│   ├── architecture.md         # System architecture
│   ├── {feature}-system.md     # Feature specifications
│   └── {feature}-implementation-plan.md
├── .tachikoma/                 # Tachikoma configuration
│   ├── config.yaml             # Backend config, policies
│   └── forge/                  # Spec Forge session artifacts
├── crates/                     # Rust crates (modular)
│   ├── tachikoma-cli-*         # CLI tools
│   ├── tachikoma-common-*      # Shared utilities
│   └── tachikoma-server-*      # Server components
├── web/                        # Frontend (SvelteKit)
├── CLAUDE.md → AGENTS.md       # Agent instructions (symlinked)
└── prompt.md                   # Current task for Ralph loop
```

---

## 4. The Ralph Loop Pattern

### 4.1 The Basic Loop
```bash
while true; do
  cat prompt.md | claude --dangerously-skip-permissions --opus4.5
done
```

Or with Tachikoma CLI (model-agnostic):
```bash
while true; do
  tachikoma run --backend=claude
done

# Or continuous mode with stop conditions:
tachikoma loop --backend=claude --stop-on=redline,test-fail
```

### 4.2 The Prompt Structure
```markdown
study specs/readme.md
study specs/{feature}-implementation-plan.md and pick the most important thing to do

IMPORTANT:
- use the tachikoma-web i18n patterns for typescript
- or the tachikoma-i18n patterns for rust
- author property based tests or unit tests (whichever is best)
- after making changes to the files run the tests
- update the implementation plan when the task is done
- when tests pass, commit and push to deploy the changes
```

### 4.3 Key Characteristics
| Aspect | Description |
|--------|-------------|
| **Control** | Low - LLM decides what to implement |
| **Oversight** | High - Human watches, steers, adjusts |
| **Goal** | Single objective per loop iteration |
| **Context** | Fresh session each iteration (no compaction) |
| **State** | File system (specs, plans, code) |
| **Deployment** | Automatic on test pass |

### 4.4 Attended vs Unattended Modes

**Attended Mode** (Learning/Debugging):
1. Run loop manually
2. Watch Tachikoma's decisions
3. Cancel if off-track
4. Adjust prompt/specs
5. Repeat

**Unattended Mode** (Production / "Night Shift"):
1. Verify specs are complete
2. Run loop in background
3. Check results periodically
4. Adjust only if needed

**Stop conditions for unattended:**
- Context redlining detected
- Repeated test failures (configurable threshold)
- No progress on plan checkboxes
- Error rate exceeds threshold

---

## 5. Specification System

### 5.1 The Pin Concept
> "The pin is my specifications. It's got my frame of reference of what this is all about."

The specs/README.md file serves as a **lookup table**:
- Links to all specifications
- Multiple descriptors per spec (improves search hit rate)
- Alternative words/synonyms for concepts
- Hints for the search tool

### 5.2 Spec File Structure
```markdown
<!--
  Copyright (c) {year} {author}. All rights reserved.
  SPDX-License-Identifier: {license}
-->
# {Feature} System Specification

**Status:** Planned | In Progress | Complete
**Version:** {semver}
**Last Updated:** {date}

## 1. Overview
{High-level description}

## 2. Requirements
{Functional and non-functional requirements}

## 3. Technical Design
{Architecture, data models, algorithms}

## 4. API Specification
{Endpoints, request/response formats}

## 5. Database Schema
{Tables, migrations, indexes}

## 6. Security Considerations
{Auth, encryption, audit}

## 7. Testing Strategy
{Unit, integration, property-based}

## 8. Migration Plan
{Rollout strategy, backwards compatibility}
```

### 5.3 Implementation Plan Structure
```markdown
# {Feature} Implementation Plan

Implementation checklist for 'specs/{feature}-system.md'. Each item cites the
relevant spec section and source code to modify.

## Phase 1: Core Types
- [ ] Create {feature}-core crate [§3.1] (pattern: crates/tachikoma-core/src/lib.rs)
- [ ] Define error types [§3.2] (pattern: crates/tachikoma-core/src/error.rs)
- [ ] Add to workspace Cargo.toml

## Phase 2: Database Schema
- [ ] Create migration XXX [§5.1]
- [ ] Define repository trait [§5.2]

## Phase 3: API Handlers
- [ ] Implement endpoints [§4]
- [ ] Add integration tests [§7]
```

### 5.4 Building Specs Through Conversation
> "I don't create my specs. I generate them. Then I review and edit them by hand."

Process:
1. Start conversation with model
2. State high-level goal
3. Let model interview you
4. Answer questions, steer direction
5. Reference existing patterns ("use the search tool")
6. Generate spec + implementation plan
7. Review and refine
8. Let Ralph loop implement

---

## 6. Spec Forge: Multi-Model Brainstorming

### 6.1 What is Spec Forge?

Spec Forge is a **multi-model brainstorming mode** where different LLMs argue, critique, and recursively improve specifications before implementation begins.

> Think of it as a "writers' room" where Tachikomas with different personalities debate the spec until it's tight.

### 6.2 Why Multi-Model?

Different models have different:
- **Training data** (different blind spots)
- **Reasoning styles** (some more cautious, some more aggressive)
- **Strengths** (Claude at nuance, Gemini at breadth, Codex at code patterns)

Combining them produces specs that are:
- More robust (edge cases caught)
- Less biased (no single model's assumptions dominate)
- Better specified (ambiguities surfaced and resolved)

### 6.3 Forge Session Structure

```
┌─────────────────────────────────────────────────────────────────┐
│                        SPEC FORGE SESSION                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Round 1: Initial Draft                                         │
│  ┌─────────┐                                                    │
│  │ Claude  │──▶ Initial spec draft                              │
│  └─────────┘                                                    │
│                                                                 │
│  Round 2: Critique & Expand                                     │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐                     │
│  │ Gemini  │    │ Codex   │    │ Claude  │                     │
│  │ Critic  │    │ Critic  │    │ Defend  │                     │
│  └────┬────┘    └────┬────┘    └────┬────┘                     │
│       │              │              │                           │
│       └──────────────┴──────────────┘                           │
│                      │                                          │
│                      ▼                                          │
│  Round 3: Synthesis                                             │
│  ┌─────────┐                                                    │
│  │ Think   │──▶ Synthesized spec (resolves conflicts)           │
│  │ Tank    │                                                    │
│  └─────────┘                                                    │
│                                                                 │
│  Round 4+: Recursive Refinement (until convergence)             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 6.4 Forge CLI Usage

```bash
# Start a Spec Forge session for a new feature
tachikoma forge --goal "Add user authentication with OAuth2" \
                --participants claude,gemini,codex \
                --oracle o3 \
                --max-rounds 5

# Resume a paused forge session
tachikoma forge --resume .tachikoma/forge/session-2026-01-11-auth.json

# Forge with human-in-the-loop after each round
tachikoma forge --goal "..." --attended
```

### 6.5 Forge Output

Each forge session produces:
1. **Spec draft** (`specs/{feature}-system.md`)
2. **Implementation plan** (`specs/{feature}-implementation-plan.md`)
3. **Decision log** (`.tachikoma/forge/{session}/decisions.md`) — rationale for each choice
4. **Dissent log** (`.tachikoma/forge/{session}/dissent.md`) — unresolved concerns

### 6.6 Convergence Detection

The forge stops when:
- All participants agree on the spec (within tolerance)
- Max rounds reached
- Human Major says "ship it"
- Circular arguments detected (no new information added)

### 6.7 Example Forge Prompts

**Round 1 (Drafter):**
```
You are drafting a specification for: {goal}

Context from existing codebase:
{retrieved patterns, conventions, related specs}

Produce a complete spec following the template in specs/TEMPLATE.md.
Be specific. Cite existing patterns where applicable.
```

**Round 2 (Critic):**
```
You are reviewing this specification draft:
{draft}

Your role: Find problems. Be adversarial but constructive.

Consider:
- Missing edge cases
- Security vulnerabilities
- Scalability concerns
- Ambiguous requirements
- Conflicts with existing patterns
- Implementation complexity

List your critiques as numbered items with severity (critical/major/minor).
```

**Round 3 (Synthesizer/Oracle):**
```
You are resolving conflicts in this specification.

Original draft:
{draft}

Critiques received:
{critiques from all participants}

Produce a revised spec that:
1. Addresses all critical/major issues
2. Documents decisions in a "Decisions" section
3. Notes any unresolved dissent in a "Known Concerns" section
```

---

## 7. File System as State / Backpressure

### 7.1 State Management
The file system IS the state machine:
- **Specs** = Requirements (immutable during implementation)
- **Implementation Plan** = Current progress (checkboxes)
- **Code** = Implementation (version controlled)
- **Tests** = Verification (gate for deployment)

### 7.2 Backpressure Mechanisms
Backpressure keeps the generative function "on the rails":

| Mechanism | Purpose |
|-----------|---------|
| Spec citations | Forces reading relevant sections |
| Pattern references | Ensures consistency with existing code |
| Test requirements | Prevents shipping broken code |
| Implementation plan | Tracks progress, prevents duplication |
| i18n patterns | Enforces localization standards |
| Error handling patterns | Consistent error types |

### 7.3 Context Window as Array
> "Context windows are a race. The less that you use in that array, the less the window needs to slide, the better outcome you get."

Optimization strategies:
1. **Fresh sessions** per task (no accumulated context rot)
2. **Lookup tables** instead of inline content
3. **Strong linkage** to search tool (let it find, don't paste)
4. **One goal per loop** (minimize context per iteration)

### 7.4 Anti-Patterns to Prevent

| Anti-Pattern | Why It's Bad | Mitigation |
|--------------|--------------|------------|
| Too many MCP tools | Bloats context, degrades quality | Cap at five primitives + oracle |
| Treating Tachikoma as Google | It's an agent, not a search engine | Train users on proper missions |
| "Secure code via prompts" | Security ≠ vibes | Sandboxing, policy, audit |
| Redlining context | Tool calls fail, quality degrades | Auto-detect and reboot |
| Mixed concerns in one context | Autoregressive failure ("gutter effect") | One mission = one context |

---

## 8. Tachikoma Architecture (Autonomous Agents)

### 8.1 What is a Tachikoma?
> "I want autonomous agents that autonomously deploy software without any code review."

Tachikomas are **autonomous coding agents** that:
- Run in isolated environments (K8s pods)
- Have full system access (sudo, deploy)
- Make decisions about implementation
- Deploy directly on test pass
- Monitor analytics/metrics
- Self-correct based on outcomes

### 8.2 Tachikoma Infrastructure
```
┌─────────────────────────────────────────────────────────────────┐
│                    TACHIKOMA INFRASTRUCTURE                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐                      ┌─────────────┐          │
│  │ K8s Cluster │                      │   DERP      │          │
│  │  (Pods)     │◀────WireGuard───────▶│   Relay     │          │
│  └──────┬──────┘                      └─────────────┘          │
│         │                                                       │
│         ▼                                                       │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│  │  SPIFFE     │    │   eBPF      │    │   Audit     │         │
│  │  Identity   │    │  Sidecar    │    │   Logs      │         │
│  └─────────────┘    └─────────────┘    └─────────────┘         │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    TACHIKOMA POD                         │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐    │   │
│  │  │ Brain   │  │  Git    │  │  Build  │  │ Deploy  │    │   │
│  │  │ (LLM)   │─▶│  Ops    │─▶│  Test   │─▶│ Push    │    │   │
│  │  └─────────┘  └─────────┘  └─────────┘  └─────────┘    │   │
│  │       │                                                  │   │
│  │       ▼                                                  │   │
│  │  ┌─────────┐                                            │   │
│  │  │ Think   │ (called as tool when needed)               │   │
│  │  │ Tank    │                                            │   │
│  │  └─────────┘                                            │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 8.3 Tachikoma Capabilities
- **Remote Execution**: K8s pod provisioning for REPL sessions
- **Identity**: SPIFFE-style identity and secret management
- **Networking**: WireGuard tunnels with DERP relay for SSH/TCP
- **Auditing**: eBPF syscall auditing sidecar
- **SCM**: Git hosting, mirroring, webhooks, branch protection
- **VCS**: jj-based version control (not git)

### 8.4 Feature Flags for Tachikomas
Tachikomas can use feature flags for safe deployments:
- Deploy with flag off
- Enable for subset of users
- Monitor analytics
- Auto-rollback if metrics degrade
- Full rollout when stable

---

## 9. Implementation Plan

### Phase 1: Core Infrastructure
- [ ] Set up project structure (specs/, crates/, web/)
- [ ] Create CLAUDE.md/AGENTS.md with base instructions
- [ ] Implement specs/README.md as lookup table
- [ ] Create spec template files
- [ ] Set up implementation plan template

### Phase 2: Ralph Loop Runner
- [ ] Create prompt.md template system
- [ ] Build loop runner script with monitoring
- [ ] Add attended/unattended mode switching
- [ ] Implement session logging
- [ ] Create alert system for failures
- [ ] Add redline detection + auto-reboot

### Phase 3: Model-Agnostic Backend
- [ ] Implement backend abstraction layer
- [ ] Add Claude backend (baseline)
- [ ] Add Codex backend
- [ ] Add Gemini CLI backend
- [ ] Add Ollama/local backend
- [ ] Implement .tachikoma/config.yaml parsing

### Phase 4: Spec Forge (Multi-Model Brainstorming)
- [ ] Implement `tachikoma forge` command
- [ ] Multi-model session orchestration
- [ ] Convergence detection algorithm
- [ ] Decision and dissent logging
- [ ] Human-in-the-loop attended mode

### Phase 5: Spec Generation System
- [ ] Build conversational spec generator
- [ ] Implement lookup table auto-update
- [ ] Create implementation plan generator
- [ ] Add citation/reference system
- [ ] Build spec validation tooling

### Phase 6: Backpressure System
- [ ] Implement pattern library
- [ ] Create i18n enforcement
- [ ] Build error handling templates
- [ ] Add test requirement gates
- [ ] Implement progress tracking

### Phase 7: Tachikoma Infrastructure
- [ ] Set up K8s cluster for pods
- [ ] Implement SPIFFE identity
- [ ] Configure WireGuard/DERP networking
- [ ] Add eBPF auditing
- [ ] Build deployment pipeline

### Phase 8: Analytics & Monitoring
- [ ] Implement PostHog-style analytics
- [ ] Build feature flag system
- [ ] Create experiment framework
- [ ] Add metric-based rollback
- [ ] Build Tachikoma dashboard

### Phase 9: Self-Evolution
- [ ] Enable Tachikomas to modify specs
- [ ] Implement spec review workflow
- [ ] Add automated testing for specs
- [ ] Build self-improvement loop
- [ ] Create knowledge base system

---

## 10. Appendix: Tachikoma Reference Architecture

### 10.1 Crate Organization (30+ crates)
```
crates/
├── tachikoma-cli-acp                    # Agent Client Protocol CLI
├── tachikoma-cli-auto-commit            # Auto-commit functionality
├── tachikoma-cli-config                 # CLI configuration
├── tachikoma-cli-credentials            # Credential management
├── tachikoma-cli-git                    # Git operations
├── tachikoma-cli-tools                  # CLI utilities
├── tachikoma-common-config              # Shared configuration
├── tachikoma-common-core                # Core types
├── tachikoma-common-http                # HTTP client
├── tachikoma-common-i18n                # Internationalization
├── tachikoma-common-secret              # Secret/PII handling
├── tachikoma-common-thread              # Threading utilities
├── tachikoma-server-api                 # API server
├── tachikoma-server-audit               # Audit logging
├── tachikoma-server-auth                # Authentication core
├── tachikoma-server-auth-devicecode     # Device code flow
├── tachikoma-server-auth-github         # GitHub OAuth
├── tachikoma-server-auth-google         # Google OAuth
├── tachikoma-server-auth-magiclink      # Magic link auth
├── tachikoma-server-auth-okta           # Okta integration
├── tachikoma-flags                      # Feature flags
├── tachikoma-flags-core                 # Flag core types
├── tachikoma-server-flags               # Flag server
├── tachikoma-analytics-core             # Analytics
├── tachikoma-forge                      # Spec Forge orchestration
├── tachikoma-backend-claude             # Claude backend
├── tachikoma-backend-codex              # Codex backend
├── tachikoma-backend-gemini             # Gemini backend
├── tachikoma-backend-ollama             # Local/Ollama backend
└── tachikoma-primitives                 # Five core tools
```

### 10.2 Spec Files (Complete List)
```
specs/
├── README.md                       # Lookup table
├── acp-system.md                   # Agent Client Protocol
├── anthropic-max-pool-management.md
├── anthropic-oauth-pool.md
├── api-documentation.md
├── architecture.md
├── audit-system.md
├── auto-commit-system.md
├── backend-abstraction.md          # NEW: Multi-backend support
├── claude-subscription-auth.md
├── configuration-system.md
├── container-system.md
├── design-system.md
├── distribution.md
├── docs-system.md
├── error-handling.md
├── feature-flags-system.md
├── git-metadata.md
├── github-app-system.md
├── health-check.md
├── i18n-system.md
├── job-scheduler-system.md
├── llm-client.md
├── spec-forge-system.md            # NEW: Multi-model brainstorming
├── streaming.md
├── analytics-system.md
└── analytics-implementation-plan.md
```

### 10.3 Key Technologies
| Component | Technology |
|-----------|------------|
| Language | Rust (backend), TypeScript (frontend) |
| Framework | Axum (API), SvelteKit (web) |
| Database | SQLite (fast iteration, then PostgreSQL) |
| VCS | jj (Jujutsu, not git) |
| Identity | SPIFFE/SPIRE |
| Networking | WireGuard + DERP relay |
| Container | K8s + Nix |
| Auditing | eBPF syscall monitoring |
| TUI | Ratatui 0.30 |
| i18n | gettext |
| CLI Framework | clap (Rust) |

### 10.4 Design Philosophy
> "Everything that exists today has been designed for humans. What would they look like if they were designed for robots?"

Questions to ask when designing:
1. Was this designed for humans? (Five Whys)
2. Can we cut it?
3. If we cut it, how do we mitigate?
4. Was it adding value?
5. What is the bare minimum the machine needs?

---

## Conclusion

The Ralph Wiggum Loop represents a paradigm shift in software development:
- **Autonomous execution** at fraction of human cost
- **Specification-driven** development
- **File system as state** for backpressure
- **High oversight, low control** management
- **Continuous deployment** with automated testing

To implement your own Tachikoma system:
1. Start with specs (build the pin)
2. Create lookup tables (improve search hit rate)
3. Write implementation plans (cite sources)
4. Use fresh sessions (avoid compaction)
5. One goal per loop (minimize context)
6. Test and deploy automatically
7. Watch, learn, adjust (attended first)
8. Scale to unattended (when confident)
9. Use Spec Forge for complex features (multi-model brainstorming)

**When Tachikoma gets confused, the answer is almost always:**
1. Reduce context
2. Reduce tools
3. Reboot (fresh context)
4. Re-anchor to the pin

> "You've got to approach this from first principles. You can do Ralph by hand because it's all about deterministically malloc'ing the array."
>
> — Geoffrey Huntley

---

*"Tachikoma on the case, Major!"* 🕷️
