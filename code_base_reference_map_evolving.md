# Codebase Reference Map (Self-Evolving)

> **AUTO-UPDATED**: Agents update this file as they implement specs.

## Crates
| Crate | Purpose |
|-------|---------|
| `tachikoma-primitives` | Core types, IDs, timestamps |
| `tachikoma-core` | Main logic, state management |
| `tachikoma-forge` | Multi-model brainstorming |

## Key Modules
- `forge/convergence/` - Agreement detection
- `forge/conflict/` - Resolution strategies
- `forge/participants/` - Model orchestration

## Patterns In Use
- `thiserror` for error types
- `tokio` async runtime
- `serde` for serialization
- `proptest` for property-based tests

## Recent Additions
<!-- AGENT: Prepend new items here, keep max 10 -->
- Phase 10 UI components (spec 250+)
- Forge panel layout
- Svelte stores + IPC bindings

---
**Update Protocol**: When you add a new crate, module, or establish a pattern used 3+ times, add 1 line here.
