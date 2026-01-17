# Tachikoma Specification Index

**The Pin** - This lookup table is the source of truth for all specifications.

## How to Use This Index

1. Find the spec you need by phase or keyword
2. Run: `cat specs/phase-XX-name/NNN-spec-name.md | claude --dangerously-skip-permissions`
3. Each spec is self-contained with dependencies, acceptance criteria, and implementation details
4. Specs are numbered for sequential building but can run in parallel within phases

## Build Order

Execute specs in numerical order. Each phase must complete before the next.

---

## Phase 0: Project Setup & Foundation (001-010)
| Spec | File | Keywords |
|------|------|----------|
| 001 | [Project Structure](phase-00-setup/001-project-structure.md) | init, scaffold, directories, workspace |
| 002 | [Rust Workspace](phase-00-setup/002-rust-workspace.md) | cargo, workspace, crates, toml |
| 003 | [Electron Shell](phase-00-setup/003-electron-shell.md) | electron, main, window, app |
| 004 | [Svelte Integration](phase-00-setup/004-svelte-integration.md) | svelte, vite, renderer, frontend |
| 005 | [IPC Bridge](phase-00-setup/005-ipc-bridge.md) | ipc, napi, native, binding |
| 005b | [IPC Listener Fix](phase-00-setup/005b-ipc-listener-fix.md) | ipc, memory, leak, listener |
| 006 | [Dev Tooling](phase-00-setup/006-dev-tooling.md) | hmr, reload, watch, development |
| 007 | [Build System](phase-00-setup/007-build-system.md) | build, compile, bundle, production |
| 008 | [Test Infrastructure](phase-00-setup/008-test-infrastructure.md) | test, jest, vitest, cargo-test |
| 009 | [CI Pipeline](phase-00-setup/009-ci-pipeline.md) | github-actions, ci, automation |
| 010 | [Documentation Setup](phase-00-setup/010-documentation-setup.md) | docs, rustdoc, typedoc |

## Phase 1: Core Common Crates (011-030)
| Spec | File | Keywords |
|------|------|----------|
| 011 | [Common Core Types](phase-01-common/011-common-core-types.md) | types, core, shared, common |
| 012 | [Error Types](phase-01-common/012-error-types.md) | error, thiserror, anyhow, handling |
| 013 | [Result Utilities](phase-01-common/013-result-utilities.md) | result, option, utilities, helpers |
| 014 | [Config Core Types](phase-01-common/014-config-core-types.md) | config, types, settings |
| 015 | [YAML Config Parsing](phase-01-common/015-yaml-config-parsing.md) | yaml, serde, parsing, config |
| 016 | [Environment Variables](phase-01-common/016-environment-variables.md) | env, dotenv, variables, secrets |
| 017 | [Secret Types](phase-01-common/017-secret-types.md) | secret, pii, redact, sensitive |
| 018 | [Thread Utilities](phase-01-common/018-thread-utilities.md) | thread, spawn, pool, concurrent |
| 019 | [Async Runtime](phase-01-common/019-async-runtime.md) | tokio, async, runtime, executor |
| 020 | [HTTP Client Foundation](phase-01-common/020-http-client-foundation.md) | http, reqwest, client, api |
| 021 | [HTTP Request Types](phase-01-common/021-http-request-types.md) | request, response, headers |
| 022 | [HTTP Retry Logic](phase-01-common/022-http-retry-logic.md) | retry, backoff, exponential |
| 023 | [i18n Core Setup](phase-01-common/023-i18n-core-setup.md) | i18n, gettext, locale, translation |
| 024 | [i18n Message Loading](phase-01-common/024-i18n-message-loading.md) | messages, po, mo, catalog |
| 025 | [i18n Locale Detection](phase-01-common/025-i18n-locale-detection.md) | locale, detect, system, preference |
| 026 | [Logging Infrastructure](phase-01-common/026-logging-infrastructure.md) | log, tracing, output, debug |
| 027 | [Tracing Setup](phase-01-common/027-tracing-setup.md) | tracing, spans, instrumentation |
| 028 | [Metrics Foundation](phase-01-common/028-metrics-foundation.md) | metrics, counters, gauges |
| 029 | [File System Utilities](phase-01-common/029-file-system-utilities.md) | fs, file, read, write, path |
| 030 | [Path Handling](phase-01-common/030-path-handling.md) | path, normalize, resolve, join |

## Phase 2: The Five Primitives (031-050)
| Spec | File | Keywords |
|------|------|----------|
| 031a | [Primitives Crate Setup](phase-02-primitives/031a-primitives-crate-setup.md) | primitives, crate, setup, features |
| 031b | [Primitives Context](phase-02-primitives/031b-primitives-context.md) | primitives, context, config, paths |
| 031c | [Primitives Results](phase-02-primitives/031c-primitives-results.md) | primitives, results, metadata, types |
| 031d | [Primitives Errors](phase-02-primitives/031d-primitives-errors.md) | primitives, errors, thiserror |
| 032 | [read_file Implementation](phase-02-primitives/032-read-file-impl.md) | read, file, contents, text |
| 033 | [read_file Error Handling](phase-02-primitives/033-read-file-errors.md) | read, error, not-found, permission |
| 034 | [list_files Implementation](phase-02-primitives/034-list-files-impl.md) | list, directory, files, entries |
| 035 | [list_files Recursive](phase-02-primitives/035-list-files-recursive.md) | recursive, walk, tree, depth |
| 036 | [bash Execution Core](phase-02-primitives/036-bash-exec-core.md) | bash, exec, command, shell |
| 037 | [bash Timeout Handling](phase-02-primitives/037-bash-timeout.md) | timeout, kill, process, duration |
| 038 | [bash Output Capture](phase-02-primitives/038-bash-output.md) | stdout, stderr, output, capture |
| 039 | [bash Error Handling](phase-02-primitives/039-bash-errors.md) | exit-code, error, failure, process |
| 040 | [edit_file Core](phase-02-primitives/040-edit-file-core.md) | edit, replace, modify, patch |
| 041 | [edit_file Uniqueness](phase-02-primitives/041-edit-file-unique.md) | unique, match, single, validation |
| 042 | [edit_file Atomic Writes](phase-02-primitives/042-edit-file-atomic.md) | atomic, write, temp, rename |
| 043 | [code_search Core](phase-02-primitives/043-code-search-core.md) | search, ripgrep, rg, pattern |
| 044 | [code_search JSON](phase-02-primitives/044-code-search-json.md) | json, parse, results, structured |
| 045 | [code_search Formatting](phase-02-primitives/045-code-search-format.md) | format, display, output, pretty |
| 046 | [Primitives Trait](phase-02-primitives/046-primitives-trait.md) | trait, interface, abstraction |
| 047 | [Primitives Validation](phase-02-primitives/047-primitives-validation.md) | validate, sanitize, check, safe |
| 047b | [Primitives Security Policy](phase-02-primitives/047b-primitives-security-policy.md) | security, bash, sandbox, policy |
| 048 | [Primitives Audit](phase-02-primitives/048-primitives-audit.md) | audit, log, trace, record |
| 049 | [Primitives Rate Limit](phase-02-primitives/049-primitives-rate-limit.md) | rate, limit, throttle, quota |
| 050 | [Primitives Tests](phase-02-primitives/050-primitives-tests.md) | test, integration, unit, coverage |

## Phase 3: Backend Abstraction Layer (051-075)
| Spec | File | Keywords |
|------|------|----------|
| 051a | [Backend Crate Setup](phase-03-backends/051a-backend-crate-setup.md) | backend, crate, setup |
| 051b | [Backend Message Types](phase-03-backends/051b-backend-message-types.md) | backend, message, role, content |
| 051c | [Backend Completion Types](phase-03-backends/051c-backend-completion-types.md) | backend, completion, request, response |
| 051d | [Backend Trait](phase-03-backends/051d-backend-trait.md) | backend, trait, interface, abstract |
| 051e | [Backend Stream Types](phase-03-backends/051e-backend-stream-types.md) | backend, stream, chunk, async |
| 052 | [Backend Config Types](phase-03-backends/052-backend-config.md) | config, backend, settings, yaml |
| 053 | [Model Role Abstraction](phase-03-backends/053-model-roles.md) | brain, think-tank, oracle, agentic |
| 054 | [Tool Definition Types](phase-03-backends/054-tool-definitions.md) | tool, definition, schema, json |
| 055 | [Tool Call Types](phase-03-backends/055-tool-call-types.md) | call, request, response, invoke |
| 056 | [Claude API Client](phase-03-backends/056-claude-api-client.md) | claude, anthropic, api, client |
| 057 | [Claude Authentication](phase-03-backends/057-claude-auth.md) | claude, auth, api-key, bearer |
| 058 | [Claude Streaming](phase-03-backends/058-claude-streaming.md) | claude, stream, sse, events |
| 059 | [Claude Tool Calling](phase-03-backends/059-claude-tools.md) | claude, tools, function, call |
| 060 | [Claude Error Handling](phase-03-backends/060-claude-errors.md) | claude, error, rate-limit, retry |
| 061 | [Codex API Client](phase-03-backends/061-codex-api-client.md) | codex, openai, api, client |
| 062 | [Codex Authentication](phase-03-backends/062-codex-auth.md) | codex, auth, api-key |
| 063 | [Codex Tool Calling](phase-03-backends/063-codex-tools.md) | codex, tools, function |
| 064 | [Gemini API Client](phase-03-backends/064-gemini-api-client.md) | gemini, google, api, client |
| 065 | [Gemini Authentication](phase-03-backends/065-gemini-auth.md) | gemini, auth, api-key |
| 066 | [Gemini Tool Calling](phase-03-backends/066-gemini-tools.md) | gemini, tools, function |
| 067 | [Ollama Local Setup](phase-03-backends/067-ollama-setup.md) | ollama, local, setup, install |
| 068 | [Ollama Model Loading](phase-03-backends/068-ollama-models.md) | ollama, model, load, pull |
| 069 | [Ollama Tool Calling](phase-03-backends/069-ollama-tools.md) | ollama, tools, function |
| 070 | [Backend Factory](phase-03-backends/070-backend-factory.md) | factory, registry, create, select |
| 071 | [Backend Health Checks](phase-03-backends/071-backend-health.md) | health, check, status, available |
| 072 | [Backend Rate Limiting](phase-03-backends/072-backend-rate-limit.md) | rate, limit, throttle, queue |
| 073 | [Backend Token Counting](phase-03-backends/073-backend-tokens.md) | token, count, estimate, budget |
| 074 | [Backend Context Tracking](phase-03-backends/074-backend-context.md) | context, window, tracking, usage |
| 075 | [Backend Tests](phase-03-backends/075-backend-tests.md) | test, mock, integration, backend |

## Phase 4: CLI Foundation (076-095)
| Spec | File | Keywords |
|------|------|----------|
| 076 | [CLI Crate Structure](phase-04-cli/076-cli-crate.md) | cli, crate, structure, clap |
| 077 | [CLI Argument Parsing](phase-04-cli/077-cli-args.md) | args, parse, flags, options |
| 078 | [CLI Subcommands](phase-04-cli/078-cli-subcommands.md) | subcommand, command, action |
| 079 | [CLI Output Formatting](phase-04-cli/079-cli-output.md) | output, format, display, terminal |
| 080 | [CLI JSON Output](phase-04-cli/080-cli-json.md) | json, output, machine, structured |
| 081 | [CLI Color Styling](phase-04-cli/081-cli-color.md) | color, ansi, style, terminal |
| 082 | [CLI Progress Indicators](phase-04-cli/082-cli-progress.md) | progress, spinner, bar, loading |
| 083 | [CLI Interactive Prompts](phase-04-cli/083-cli-prompts.md) | prompt, interactive, input, confirm |
| 084 | [CLI config Command](phase-04-cli/084-cli-config-cmd.md) | config, view, edit, command |
| 085 | [CLI doctor Command](phase-04-cli/085-cli-doctor-cmd.md) | doctor, diagnose, check, health |
| 086 | [CLI tools Command](phase-04-cli/086-cli-tools-cmd.md) | tools, list, verify, primitives |
| 087 | [CLI backends Command](phase-04-cli/087-cli-backends-cmd.md) | backends, list, available, models |
| 088 | [CLI init Scaffolding](phase-04-cli/088-cli-init-scaffold.md) | init, scaffold, create, project |
| 089 | [CLI init Templates](phase-04-cli/089-cli-init-templates.md) | template, generate, boilerplate |
| 090 | [CLI Help System](phase-04-cli/090-cli-help.md) | help, usage, manual, docs |
| 091 | [CLI Error Messages](phase-04-cli/091-cli-errors.md) | error, message, helpful, recovery |
| 092 | [CLI Logging Integration](phase-04-cli/092-cli-logging.md) | log, verbose, debug, trace |
| 093 | [CLI Shell Completions](phase-04-cli/093-cli-completions.md) | completion, bash, zsh, fish |
| 094 | [CLI Man Pages](phase-04-cli/094-cli-man.md) | man, page, documentation |
| 095 | [CLI Tests](phase-04-cli/095-cli-tests.md) | test, cli, integration, e2e |

## Phase 5: Ralph Loop Runner (096-115)
| Spec | File | Keywords |
|------|------|----------|
| 096a | [Loop Crate Setup](phase-05-loop/096a-loop-crate-setup.md) | loop, crate, setup, id |
| 096b | [Loop Config](phase-05-loop/096b-loop-config.md) | loop, config, settings, stop |
| 096c | [Loop State](phase-05-loop/096c-loop-state.md) | loop, state, stats, context |
| 096d | [Loop Runner](phase-05-loop/096d-loop-runner.md) | loop, runner, command, execute |
| 097 | [Loop Iteration Logic](phase-05-loop/097-loop-iteration.md) | iteration, cycle, repeat, next |
| 098 | [Prompt Loading](phase-05-loop/098-prompt-loading.md) | prompt, load, read, prompt.md |
| 099 | [Prompt Templates](phase-05-loop/099-prompt-templates.md) | template, variable, substitute |
| 100 | [Session Management](phase-05-loop/100-session-management.md) | session, manage, create, destroy |
| 101 | [Fresh Context Creation](phase-05-loop/101-fresh-context.md) | fresh, context, new, clean |
| 102 | [Context Redline Detection](phase-05-loop/102-redline-detection.md) | redline, detect, threshold, warning |
| 103 | [Auto-Reboot Logic](phase-05-loop/103-auto-reboot.md) | reboot, auto, restart, fresh |
| 104 | [Stop Condition Types](phase-05-loop/104-stop-conditions.md) | stop, condition, halt, terminate |
| 105 | [Stop Condition Evaluation](phase-05-loop/105-stop-evaluation.md) | evaluate, check, stop, trigger |
| 106 | [Test Failure Tracking](phase-05-loop/106-test-failure-tracking.md) | test, fail, streak, count |
| 107 | [No Progress Detection](phase-05-loop/107-no-progress.md) | progress, stuck, stall, detect |
| 108 | [Loop Metrics](phase-05-loop/108-loop-metrics.md) | metrics, stats, count, measure |
| 109 | [Loop State Persistence](phase-05-loop/109-loop-state.md) | state, persist, save, restore |
| 110 | [Attended Mode](phase-05-loop/110-attended-mode.md) | attended, watch, pause, approve |
| 111 | [Unattended Mode](phase-05-loop/111-unattended-mode.md) | unattended, autonomous, night-shift |
| 112 | [Mode Switching](phase-05-loop/112-mode-switching.md) | switch, mode, toggle, change |
| 113 | [Loop Event Hooks](phase-05-loop/113-loop-hooks.md) | hook, event, callback, trigger |
| 114 | [Loop Notifications](phase-05-loop/114-loop-notifications.md) | notify, alert, email, slack |
| 115 | [Loop Tests](phase-05-loop/115-loop-tests.md) | test, loop, integration, mock |

## Phase 6: Spec System (116-135)
| Spec | File | Keywords |
|------|------|----------|
| 116 | [Spec Directory Structure](phase-06-specs/116-spec-directory.md) | directory, structure, organize |
| 117 | [Spec Template Types](phase-06-specs/117-spec-templates.md) | template, type, format, structure |
| 118 | [README Lookup Format](phase-06-specs/118-readme-lookup.md) | readme, lookup, index, table |
| 119 | [README Auto-Generation](phase-06-specs/119-readme-autogen.md) | generate, auto, readme, update |
| 120 | [Spec File Parsing](phase-06-specs/120-spec-parsing.md) | parse, markdown, extract, frontmatter |
| 121 | [Spec Metadata Extraction](phase-06-specs/121-spec-metadata.md) | metadata, status, version, date |
| 122 | [Implementation Plan Format](phase-06-specs/122-impl-plan-format.md) | plan, format, checkbox, phase |
| 123 | [Checkbox Tracking](phase-06-specs/123-checkbox-tracking.md) | checkbox, track, complete, progress |
| 123b | [Spec Status Sync](phase-06-specs/123b-spec-status-sync.md) | status, sync, checkbox, auto-update |
| 124 | [Progress Calculation](phase-06-specs/124-progress-calc.md) | progress, percent, calculate, done |
| 125 | [Spec Citation System](phase-06-specs/125-spec-citation.md) | citation, reference, section, link |
| 126 | [Pattern Reference Linking](phase-06-specs/126-pattern-linking.md) | pattern, link, reference, code |
| 127 | [Spec Validation Rules](phase-06-specs/127-spec-validation.md) | validate, rule, check, lint |
| 128 | [Spec Linting](phase-06-specs/128-spec-linting.md) | lint, style, format, consistent |
| 129 | [Spec Versioning](phase-06-specs/129-spec-versioning.md) | version, semver, history, track |
| 130 | [Spec Diff Generation](phase-06-specs/130-spec-diff.md) | diff, compare, change, delta |
| 131 | [Spec Search Indexing](phase-06-specs/131-spec-search-index.md) | search, index, keyword, fulltext |
| 132 | [Spec Search API](phase-06-specs/132-spec-search-api.md) | search, api, query, results |
| 133 | [Spec Template Rendering](phase-06-specs/133-spec-rendering.md) | render, template, generate |
| 134 | [Spec Generation Conversation](phase-06-specs/134-spec-conversation.md) | conversation, generate, interview |
| 135 | [Spec System Tests](phase-06-specs/135-spec-tests.md) | test, spec, integration |

## Phase 7: Spec Forge - Multi-Model Brainstorming (136-160)
| Spec | File | Keywords |
|------|------|----------|
| 136a | [Forge Crate Setup](phase-07-forge/136a-forge-crate-setup.md) | forge, crate, setup |
| 136b | [Forge Session Types](phase-07-forge/136b-forge-session-types.md) | forge, session, status, topic |
| 136c | [Forge Round Types](phase-07-forge/136c-forge-round-types.md) | forge, round, draft, critique |
| 136d | [Forge Participant Types](phase-07-forge/136d-forge-participant-types.md) | forge, participant, model, response |
| 137 | [Forge Configuration](phase-07-forge/137-forge-config.md) | config, forge, settings, yaml |
| 138 | [Forge Participants](phase-07-forge/138-forge-participants.md) | participant, model, backend, role |
| 139 | [Forge Round Orchestration](phase-07-forge/139-forge-rounds.md) | round, orchestrate, sequence |
| 140 | [Round 1 Initial Draft](phase-07-forge/140-round1-draft.md) | draft, initial, first, create |
| 141 | [Round 2 Critique Prompts](phase-07-forge/141-round2-critique-prompts.md) | critique, prompt, template |
| 142 | [Round 2 Critique Collection](phase-07-forge/142-round2-critique-collect.md) | collect, critique, gather, merge |
| 143 | [Round 3 Synthesis Prompts](phase-07-forge/143-round3-synthesis-prompts.md) | synthesis, prompt, template |
| 144 | [Round 3 Conflict Resolution](phase-07-forge/144-round3-conflict.md) | conflict, resolve, merge, decide |
| 145 | [Recursive Refinement](phase-07-forge/145-recursive-refine.md) | recursive, refine, iterate, loop |
| 146 | [Convergence Detection](phase-07-forge/146-convergence-detect.md) | convergence, detect, stable, done |
| 147 | [Convergence Metrics](phase-07-forge/147-convergence-metrics.md) | metrics, measure, convergence |
| 148 | [Decision Logging](phase-07-forge/148-decision-logging.md) | decision, log, record, rationale |
| 149 | [Dissent Logging](phase-07-forge/149-dissent-logging.md) | dissent, concern, unresolved, log |
| 150 | [Forge Session Persistence](phase-07-forge/150-forge-persistence.md) | persist, save, session, state |
| 151 | [Forge Session Resume](phase-07-forge/151-forge-resume.md) | resume, continue, restore, session |
| 152 | [Forge Attended Mode](phase-07-forge/152-forge-attended.md) | attended, human, approve, review |
| 153 | [Forge CLI Command](phase-07-forge/153-forge-cli.md) | cli, command, forge, tachikoma |
| 154 | [Forge Output Generation](phase-07-forge/154-forge-output.md) | output, generate, spec, plan |
| 155 | [Forge Timeout Handling](phase-07-forge/155-forge-timeout.md) | timeout, limit, max-time, abort |
| 156 | [Forge Cost Tracking](phase-07-forge/156-forge-cost.md) | cost, token, expense, budget |
| 157 | [Forge Quality Metrics](phase-07-forge/157-forge-quality.md) | quality, score, evaluate, grade |
| 158 | [Forge Templates](phase-07-forge/158-forge-templates.md) | template, prompt, format |
| 159 | [Forge Result Validation](phase-07-forge/159-forge-validation.md) | validate, result, check, verify |
| 160 | [Forge Tests](phase-07-forge/160-forge-tests.md) | test, forge, integration, mock |

## Phase 8: Electron Shell (161-185)
| Spec | File | Keywords |
|------|------|----------|
| 161 | [Electron Main Process](phase-08-electron/161-electron-main.md) | main, process, electron, entry |
| 162 | [Window Management](phase-08-electron/162-window-management.md) | window, create, manage, multi |
| 163 | [Menu System](phase-08-electron/163-menu-system.md) | menu, bar, context, shortcut |
| 164 | [Native Dialogs](phase-08-electron/164-native-dialogs.md) | dialog, open, save, message |
| 165 | [File System Access](phase-08-electron/165-fs-access.md) | file, system, access, permission |
| 166 | [App Lifecycle](phase-08-electron/166-app-lifecycle.md) | lifecycle, ready, quit, activate |
| 167 | [Auto Updates](phase-08-electron/167-auto-updates.md) | update, auto, check, download |
| 168 | [Crash Reporting](phase-08-electron/168-crash-reporting.md) | crash, report, sentry, error |
| 169 | [Security Config](phase-08-electron/169-security-config.md) | security, csp, sandbox, nodeIntegration |
| 170 | [IPC Channels](phase-08-electron/170-ipc-channels.md) | ipc, channel, main, renderer |
| 171 | [Preload Scripts](phase-08-electron/171-preload-scripts.md) | preload, script, bridge, expose |
| 172 | [Context Bridge](phase-08-electron/172-context-bridge.md) | context, bridge, api, expose |
| 173 | [Rust Native Binding](phase-08-electron/173-rust-native.md) | rust, native, napi, binding |
| 174 | [NAPI-RS Setup](phase-08-electron/174-napi-rs-setup.md) | napi, rs, setup, build |
| 175 | [Build Configuration](phase-08-electron/175-build-config.md) | build, electron-builder, config |
| 176 | [Code Signing](phase-08-electron/176-code-signing.md) | sign, certificate, identity |
| 177 | [macOS Build](phase-08-electron/177-macos-build.md) | macos, dmg, app, darwin |
| 178 | [Windows Build](phase-08-electron/178-windows-build.md) | windows, exe, installer, win32 |
| 179 | [Linux Build](phase-08-electron/179-linux-build.md) | linux, appimage, deb, rpm |
| 180 | [DevTools Integration](phase-08-electron/180-devtools.md) | devtools, debug, inspect, console |
| 181 | [Deep Linking](phase-08-electron/181-deep-linking.md) | deep, link, protocol, url |
| 182 | [Protocol Handlers](phase-08-electron/182-protocol-handlers.md) | protocol, handler, tachikoma:// |
| 183 | [Tray Integration](phase-08-electron/183-tray-integration.md) | tray, icon, menu, system |
| 184 | [Notifications](phase-08-electron/184-notifications.md) | notification, native, alert, toast |
| 185 | [Electron Tests](phase-08-electron/185-electron-tests.md) | test, electron, spectron, e2e |

## Phase 9: Svelte UI Foundation (186-215)
| Spec | File | Keywords |
|------|------|----------|
| 186 | [SvelteKit Setup](phase-09-ui-foundation/186-sveltekit-setup.md) | svelte, kit, setup, project |
| 187 | [Routing Configuration](phase-09-ui-foundation/187-routing-config.md) | route, page, navigation, spa |
| 188 | [Layout System](phase-09-ui-foundation/188-layout-system.md) | layout, slot, nested, structure |
| 189 | [Store Architecture](phase-09-ui-foundation/189-store-architecture.md) | store, state, writable, derived |
| 190 | [IPC Store Bindings](phase-09-ui-foundation/190-ipc-store-bindings.md) | ipc, store, bind, sync |
| 190b | [Glassmorphic Design System](phase-09-ui-foundation/190b-glassmorphic-design-system.md) | glass, blur, translucent, modern |
| 191 | [Design Tokens](phase-09-ui-foundation/191-design-tokens.md) | token, design, css, variable |
| 192 | [Typography System](phase-09-ui-foundation/192-typography.md) | font, typography, text, heading |
| 193 | [Color System](phase-09-ui-foundation/193-color-system.md) | color, palette, theme, tachikoma |
| 194 | [Spacing System](phase-09-ui-foundation/194-spacing-system.md) | spacing, margin, padding, gap |
| 195 | [Shadows & Elevation](phase-09-ui-foundation/195-shadows-elevation.md) | shadow, elevation, depth, layer |
| 196 | [Component Library Setup](phase-09-ui-foundation/196-component-library.md) | component, library, setup, storybook |
| 197 | [Button Component](phase-09-ui-foundation/197-button-component.md) | button, click, action, primary |
| 198 | [Input Component](phase-09-ui-foundation/198-input-component.md) | input, text, field, form |
| 199 | [Select Component](phase-09-ui-foundation/199-select-component.md) | select, dropdown, option, choice |
| 200 | [Checkbox Toggle](phase-09-ui-foundation/200-checkbox-toggle.md) | checkbox, toggle, switch, boolean |
| 201 | [Card Component](phase-09-ui-foundation/201-card-component.md) | card, container, box, panel |
| 202 | [Modal Component](phase-09-ui-foundation/202-modal-component.md) | modal, dialog, overlay, popup |
| 203 | [Toast Component](phase-09-ui-foundation/203-toast-component.md) | toast, notification, snackbar |
| 204 | [Tooltip Component](phase-09-ui-foundation/204-tooltip-component.md) | tooltip, hover, hint, info |
| 205 | [Tabs Component](phase-09-ui-foundation/205-tabs-component.md) | tab, panel, switch, navigation |
| 206 | [Accordion Component](phase-09-ui-foundation/206-accordion-component.md) | accordion, collapse, expand, section |
| 207 | [Tree View Component](phase-09-ui-foundation/207-tree-view.md) | tree, view, hierarchy, nested |
| 208 | [Code Block Component](phase-09-ui-foundation/208-code-block.md) | code, block, syntax, highlight |
| 209 | [Diff Viewer Component](phase-09-ui-foundation/209-diff-viewer.md) | diff, view, compare, change |
| 210 | [Terminal Component](phase-09-ui-foundation/210-terminal-component.md) | terminal, emulator, console, shell |
| 211 | [Loading States](phase-09-ui-foundation/211-loading-states.md) | loading, skeleton, spinner, wait |
| 212 | [Error Boundaries](phase-09-ui-foundation/212-error-boundaries.md) | error, boundary, catch, fallback |
| 213 | [Form Validation](phase-09-ui-foundation/213-form-validation.md) | form, validate, error, submit |
| 214 | [Keyboard Shortcuts](phase-09-ui-foundation/214-keyboard-shortcuts.md) | keyboard, shortcut, hotkey, bind |
| 215 | [UI Component Tests](phase-09-ui-foundation/215-ui-tests.md) | test, component, vitest, testing-library |

## Phase 10: Mission Panel UI (216-235)
| Spec | File | Keywords |
|------|------|----------|
| 216 | [Mission Panel Layout](phase-10-mission-ui/216-mission-layout.md) | mission, panel, layout, structure |
| 217 | [Mission State Management](phase-10-mission-ui/217-mission-state.md) | mission, state, store, manage |
| 218 | [Mission Creation Flow](phase-10-mission-ui/218-mission-creation.md) | create, mission, new, wizard |
| 219 | [Mission Prompt Editor](phase-10-mission-ui/219-prompt-editor.md) | prompt, editor, markdown, text |
| 220 | [Mission Spec Selector](phase-10-mission-ui/220-spec-selector.md) | spec, selector, choose, pick |
| 221 | [Mission Backend Selector](phase-10-mission-ui/221-backend-selector.md) | backend, selector, model, choose |
| 222 | [Mission Mode Toggle](phase-10-mission-ui/222-mode-toggle.md) | mode, toggle, attended, unattended |
| 223 | [Mission Controls](phase-10-mission-ui/223-mission-controls.md) | control, start, stop, pause |
| 224 | [Mission Progress Display](phase-10-mission-ui/224-progress-display.md) | progress, display, status, bar |
| 225 | [Mission Log Viewer](phase-10-mission-ui/225-log-viewer.md) | log, viewer, stream, tail |
| 226 | [Mission Checkpoint Display](phase-10-mission-ui/226-checkpoint-display.md) | checkpoint, display, step, approve |
| 227 | [Mission Diff Preview](phase-10-mission-ui/227-diff-preview.md) | diff, preview, change, code |
| 228 | [Mission Test Results](phase-10-mission-ui/228-test-results.md) | test, result, pass, fail |
| 229 | [Mission Cost Tracking](phase-10-mission-ui/229-cost-tracking.md) | cost, token, expense, display |
| 230 | [Mission Context Meter](phase-10-mission-ui/230-context-meter.md) | context, meter, usage, gauge |
| 231 | [Mission Redline Warning](phase-10-mission-ui/231-redline-warning.md) | redline, warning, alert, danger |
| 232 | [Mission History View](phase-10-mission-ui/232-history-view.md) | history, past, list, archive |
| 233 | [Mission Comparison](phase-10-mission-ui/233-mission-comparison.md) | compare, mission, diff, side |
| 234 | [Mission Export](phase-10-mission-ui/234-mission-export.md) | export, download, json, share |
| 235 | [Mission Panel Tests](phase-10-mission-ui/235-mission-tests.md) | test, mission, panel, e2e |

## Phase 11: Spec Browser UI (236-255)
| Spec | File | Keywords |
|------|------|----------|
| 236 | [Spec Browser Layout](phase-11-spec-browser/236-spec-browser-layout.md) | browser, layout, spec, panel |
| 237 | [Spec Tree Navigation](phase-11-spec-browser/237-spec-tree-nav.md) | tree, navigation, folder, file |
| 238 | [Spec File Viewer](phase-11-spec-browser/238-spec-file-viewer.md) | file, viewer, markdown, read |
| 239 | [Spec Markdown Renderer](phase-11-spec-browser/239-markdown-renderer.md) | markdown, render, html, display |
| 240 | [Spec Editor Integration](phase-11-spec-browser/240-spec-editor.md) | editor, edit, modify, save |
| 241 | [Impl Plan Viewer](phase-11-spec-browser/241-impl-plan-viewer.md) | plan, view, checkbox, progress |
| 242 | [Impl Plan Checkbox](phase-11-spec-browser/242-impl-checkbox.md) | checkbox, click, toggle, update |
| 243 | [Spec Search UI](phase-11-spec-browser/243-spec-search-ui.md) | search, ui, input, query |
| 244 | [Spec Search Results](phase-11-spec-browser/244-spec-search-results.md) | results, list, match, highlight |
| 245 | [Spec Quick Nav](phase-11-spec-browser/245-spec-quick-nav.md) | quick, nav, jump, goto |
| 246 | [Spec Breadcrumbs](phase-11-spec-browser/246-spec-breadcrumbs.md) | breadcrumb, path, trail, navigate |
| 247 | [Spec Metadata Panel](phase-11-spec-browser/247-spec-metadata.md) | metadata, panel, info, details |
| 248 | [Spec Version History](phase-11-spec-browser/248-spec-version-history.md) | version, history, git, log |
| 249 | [Spec Diff Viewer](phase-11-spec-browser/249-spec-diff-viewer.md) | diff, view, compare, version |
| 250 | [Spec Creation Wizard](phase-11-spec-browser/250-spec-creation.md) | create, wizard, new, spec |
| 251 | [Spec Template Selection](phase-11-spec-browser/251-template-selection.md) | template, select, choose, type |
| 252 | [Spec Validation UI](phase-11-spec-browser/252-spec-validation-ui.md) | validate, ui, error, warning |
| 253 | [Spec Linking UI](phase-11-spec-browser/253-spec-linking.md) | link, reference, connect, relate |
| 254 | [Spec Export Options](phase-11-spec-browser/254-spec-export.md) | export, download, pdf, html |
| 255 | [Spec Browser Tests](phase-11-spec-browser/255-spec-browser-tests.md) | test, browser, spec, e2e |

## Phase 12: Spec Forge UI (256-275)
| Spec | File | Keywords |
|------|------|----------|
| 256 | [Forge Panel Layout](phase-12-forge-ui/256-forge-layout.md) | forge, panel, layout, structure |
| 257 | [Forge Session Creation](phase-12-forge-ui/257-session-creation.md) | session, create, new, start |
| 258 | [Forge Goal Input](phase-12-forge-ui/258-goal-input.md) | goal, input, describe, objective |
| 259 | [Forge Participant Selection](phase-12-forge-ui/259-participant-select.md) | participant, select, model, add |
| 260 | [Forge Oracle Selection](phase-12-forge-ui/260-oracle-select.md) | oracle, select, think-tank, choose |
| 261 | [Forge Round Visualization](phase-12-forge-ui/261-round-visualization.md) | round, visualize, timeline, step |
| 262 | [Forge Draft Viewer](phase-12-forge-ui/262-draft-viewer.md) | draft, view, spec, current |
| 263 | [Forge Critique Viewer](phase-12-forge-ui/263-critique-viewer.md) | critique, view, feedback, list |
| 264 | [Forge Conflict Highlights](phase-12-forge-ui/264-conflict-highlights.md) | conflict, highlight, mark, diff |
| 265 | [Forge Decision Log UI](phase-12-forge-ui/265-decision-log-ui.md) | decision, log, view, rationale |
| 266 | [Forge Dissent Log UI](phase-12-forge-ui/266-dissent-log-ui.md) | dissent, log, view, concern |
| 267 | [Forge Convergence Indicator](phase-12-forge-ui/267-convergence-indicator.md) | convergence, indicator, progress |
| 268 | [Forge Session Controls](phase-12-forge-ui/268-session-controls.md) | control, pause, resume, stop |
| 269 | [Forge Pause Resume](phase-12-forge-ui/269-pause-resume.md) | pause, resume, continue, wait |
| 270 | [Forge Human Intervention](phase-12-forge-ui/270-human-intervention.md) | human, intervention, approve, edit |
| 271 | [Forge Result Preview](phase-12-forge-ui/271-result-preview.md) | result, preview, final, spec |
| 272 | [Forge Result Acceptance](phase-12-forge-ui/272-result-acceptance.md) | accept, result, approve, finalize |
| 273 | [Forge History Browser](phase-12-forge-ui/273-history-browser.md) | history, browse, past, session |
| 274 | [Forge Comparison View](phase-12-forge-ui/274-comparison-view.md) | compare, view, session, diff |
| 275 | [Forge UI Tests](phase-12-forge-ui/275-forge-ui-tests.md) | test, forge, ui, e2e |

## Phase 13: Settings & Configuration UI (276-295)
| Spec | File | Keywords |
|------|------|----------|
| 276 | [Settings Panel Layout](phase-13-settings/276-settings-layout.md) | settings, panel, layout, structure |
| 277 | [Backend Config UI](phase-13-settings/277-backend-config-ui.md) | backend, config, ui, settings |
| 278 | [Brain Model Selection](phase-13-settings/278-brain-selection.md) | brain, model, select, agentic |
| 279 | [Think Tank Selection](phase-13-settings/279-think-tank-selection.md) | think-tank, oracle, select, model |
| 280 | [API Key Management](phase-13-settings/280-api-key-management.md) | api, key, manage, secure, input |
| 281 | [Loop Config UI](phase-13-settings/281-loop-config-ui.md) | loop, config, settings, options |
| 282 | [Stop Conditions UI](phase-13-settings/282-stop-conditions-ui.md) | stop, condition, configure, set |
| 283 | [Policy Config UI](phase-13-settings/283-policy-config-ui.md) | policy, config, rules, settings |
| 284 | [Theme Selection](phase-13-settings/284-theme-selection.md) | theme, select, dark, light, tachikoma |
| 285 | [Font Accessibility](phase-13-settings/285-font-accessibility.md) | font, size, accessibility, zoom |
| 286 | [Keyboard Config](phase-13-settings/286-keyboard-config.md) | keyboard, shortcut, customize, bind |
| 287 | [Notification Preferences](phase-13-settings/287-notification-prefs.md) | notification, preference, alert, sound |
| 288 | [Data Cache Management](phase-13-settings/288-data-cache.md) | data, cache, clear, storage |
| 289 | [Export Import Settings](phase-13-settings/289-export-import.md) | export, import, settings, backup |
| 290 | [Profile Management](phase-13-settings/290-profile-management.md) | profile, manage, switch, user |
| 291 | [Workspace Settings](phase-13-settings/291-workspace-settings.md) | workspace, settings, project, local |
| 292 | [Git Integration Settings](phase-13-settings/292-git-settings.md) | git, integration, settings, config |
| 293 | [Telemetry Preferences](phase-13-settings/293-telemetry-prefs.md) | telemetry, opt-in, opt-out, privacy |
| 294 | [Update Preferences](phase-13-settings/294-update-prefs.md) | update, preference, auto, check |
| 295 | [Settings Tests](phase-13-settings/295-settings-tests.md) | test, settings, ui, e2e |

## Phase 14: Dashboard & Analytics UI (296-315)
| Spec | File | Keywords |
|------|------|----------|
| 296 | [Dashboard Layout](phase-14-dashboard/296-dashboard-layout.md) | dashboard, layout, structure, home |
| 297 | [Mission Overview Cards](phase-14-dashboard/297-mission-cards.md) | mission, card, overview, summary |
| 298 | [Active Mission Status](phase-14-dashboard/298-active-status.md) | active, mission, status, current |
| 299 | [Recent Missions List](phase-14-dashboard/299-recent-missions.md) | recent, mission, list, history |
| 300 | [Cost Summary Widget](phase-14-dashboard/300-cost-summary.md) | cost, summary, widget, expense |
| 301 | [Token Usage Charts](phase-14-dashboard/301-token-charts.md) | token, usage, chart, graph |
| 302 | [Success Rate Metrics](phase-14-dashboard/302-success-rate.md) | success, rate, metric, percent |
| 303 | [Time Series Graphs](phase-14-dashboard/303-time-series.md) | time, series, graph, trend |
| 304 | [Context Usage Visualization](phase-14-dashboard/304-context-viz.md) | context, usage, visualize, gauge |
| 305 | [Test Pass Fail Charts](phase-14-dashboard/305-test-charts.md) | test, pass, fail, chart |
| 306 | [Deploy Frequency Metrics](phase-14-dashboard/306-deploy-metrics.md) | deploy, frequency, metric, count |
| 307 | [Error Rate Tracking](phase-14-dashboard/307-error-rate.md) | error, rate, track, trend |
| 308 | [Performance Trending](phase-14-dashboard/308-performance-trend.md) | performance, trend, speed, latency |
| 309 | [Daily Weekly Summaries](phase-14-dashboard/309-summaries.md) | daily, weekly, summary, report |
| 310 | [Export Reports](phase-14-dashboard/310-export-reports.md) | export, report, pdf, csv |
| 311 | [Dashboard Filters](phase-14-dashboard/311-dashboard-filters.md) | filter, dashboard, select, narrow |
| 312 | [Dashboard Date Ranges](phase-14-dashboard/312-date-ranges.md) | date, range, picker, period |
| 313 | [Dashboard Refresh](phase-14-dashboard/313-dashboard-refresh.md) | refresh, reload, auto, manual |
| 314 | [Real-time Updates](phase-14-dashboard/314-realtime-updates.md) | realtime, update, websocket, live |
| 315 | [Dashboard Tests](phase-14-dashboard/315-dashboard-tests.md) | test, dashboard, ui, e2e |

## Phase 15: Server Infrastructure (316-340)
| Spec | File | Keywords |
|------|------|----------|
| 316 | [Server Crate Structure](phase-15-server/316-server-crate.md) | server, crate, structure, axum |
| 317 | [Axum Router Setup](phase-15-server/317-axum-router.md) | axum, router, route, handler |
| 318 | [API Versioning](phase-15-server/318-api-versioning.md) | api, version, v1, prefix |
| 319 | [Request Response Types](phase-15-server/319-request-response.md) | request, response, type, dto |
| 320 | [Request Validation](phase-15-server/320-request-validation.md) | validate, request, schema, check |
| 321 | [Error Response Format](phase-15-server/321-error-response.md) | error, response, format, json |
| 322 | [Authentication Middleware](phase-15-server/322-auth-middleware.md) | auth, middleware, jwt, bearer |
| 323 | [Authorization Middleware](phase-15-server/323-authz-middleware.md) | authz, middleware, permission, role |
| 324 | [Rate Limiting Middleware](phase-15-server/324-rate-limit-mw.md) | rate, limit, middleware, throttle |
| 325 | [Logging Middleware](phase-15-server/325-logging-middleware.md) | log, middleware, request, trace |
| 326 | [CORS Configuration](phase-15-server/326-cors-config.md) | cors, config, origin, header |
| 327 | [Health Check Endpoints](phase-15-server/327-health-endpoints.md) | health, check, endpoint, status |
| 328 | [Metrics Endpoints](phase-15-server/328-metrics-endpoints.md) | metrics, endpoint, prometheus |
| 329 | [WebSocket Setup](phase-15-server/329-websocket-setup.md) | websocket, setup, upgrade, ws |
| 330 | [WebSocket Message Types](phase-15-server/330-ws-message-types.md) | websocket, message, type, json |
| 331 | [WebSocket Connection Mgmt](phase-15-server/331-ws-connection.md) | websocket, connection, manage |
| 332 | [Server Configuration](phase-15-server/332-server-config.md) | server, config, settings, yaml |
| 333 | [Server Startup](phase-15-server/333-server-startup.md) | server, startup, init, boot |
| 334 | [Graceful Shutdown](phase-15-server/334-graceful-shutdown.md) | graceful, shutdown, signal, stop |
| 335 | [TLS Configuration](phase-15-server/335-tls-config.md) | tls, ssl, https, certificate |
| 336 | [Server Monitoring](phase-15-server/336-server-monitoring.md) | monitor, server, health, alert |
| 337 | [Server Scaling](phase-15-server/337-server-scaling.md) | scale, horizontal, vertical, load |
| 338 | [Server Caching](phase-15-server/338-server-caching.md) | cache, server, redis, memory |
| 339 | [Database Connection](phase-15-server/339-db-connection.md) | database, connection, pool, server |
| 340 | [Server Tests](phase-15-server/340-server-tests.md) | test, server, integration, e2e |

## Phase 16: Database Layer (341-365)
| Spec | File | Keywords |
|------|------|----------|
| 341 | [Database Connection Pool](phase-16-database/341-db-pool.md) | pool, connection, database, manage |
| 342 | [SQLite Configuration](phase-16-database/342-sqlite-config.md) | sqlite, config, database, local |
| 343 | [Migration System](phase-16-database/343-migration-system.md) | migration, system, schema, evolve |
| 344 | [Migration CLI](phase-16-database/344-migration-cli.md) | migration, cli, run, create |
| 345 | [Mission Table Schema](phase-16-database/345-mission-schema.md) | mission, table, schema, columns |
| 346 | [Mission Repository](phase-16-database/346-mission-repo.md) | mission, repository, crud, query |
| 347 | [Spec Table Schema](phase-16-database/347-spec-schema.md) | spec, table, schema, columns |
| 348 | [Spec Repository](phase-16-database/348-spec-repo.md) | spec, repository, crud, query |
| 349 | [Forge Session Schema](phase-16-database/349-forge-schema.md) | forge, session, table, schema |
| 350 | [Forge Session Repository](phase-16-database/350-forge-repo.md) | forge, repository, crud, query |
| 351 | [Config Table Schema](phase-16-database/351-config-schema.md) | config, table, schema, settings |
| 352 | [Config Repository](phase-16-database/352-config-repo.md) | config, repository, crud, query |
| 353 | [Audit Log Schema](phase-16-database/353-audit-schema.md) | audit, log, table, schema |
| 354 | [Audit Log Repository](phase-16-database/354-audit-repo.md) | audit, repository, crud, query |
| 355 | [Analytics Event Schema](phase-16-database/355-analytics-schema.md) | analytics, event, table, schema |
| 356 | [Analytics Event Repository](phase-16-database/356-analytics-repo.md) | analytics, repository, crud, query |
| 357 | [User Preferences Schema](phase-16-database/357-user-prefs-schema.md) | user, preferences, table, schema |
| 358 | [User Preferences Repository](phase-16-database/358-user-prefs-repo.md) | user, preferences, repository |
| 359 | [Query Builders](phase-16-database/359-query-builders.md) | query, builder, sql, construct |
| 360 | [Transaction Handling](phase-16-database/360-transactions.md) | transaction, commit, rollback |
| 361 | [Database Backup](phase-16-database/361-db-backup.md) | backup, database, export, save |
| 362 | [Database Restore](phase-16-database/362-db-restore.md) | restore, database, import, load |
| 363 | [Database Vacuuming](phase-16-database/363-db-vacuum.md) | vacuum, optimize, compact, clean |
| 364 | [PostgreSQL Migration](phase-16-database/364-postgres-migration.md) | postgres, migration, upgrade, path |
| 365 | [Database Tests](phase-16-database/365-db-tests.md) | test, database, integration |

## Phase 17: Authentication System (366-390)
| Spec | File | Keywords |
|------|------|----------|
| 366 | [Auth Core Types](phase-17-auth/366-auth-core-types.md) | auth, core, type, struct |
| 367 | [Auth Session Management](phase-17-auth/367-auth-session.md) | session, manage, create, destroy |
| 368 | [Auth Token Types](phase-17-auth/368-auth-tokens.md) | token, type, access, refresh |
| 369 | [JWT Implementation](phase-17-auth/369-jwt-impl.md) | jwt, implement, sign, verify |
| 370 | [Refresh Token Handling](phase-17-auth/370-refresh-token.md) | refresh, token, rotate, expire |
| 371 | [Device Code Flow Types](phase-17-auth/371-device-code-types.md) | device, code, flow, type |
| 372 | [Device Code Flow Impl](phase-17-auth/372-device-code-impl.md) | device, code, implement, poll |
| 373 | [GitHub OAuth Config](phase-17-auth/373-github-oauth-config.md) | github, oauth, config, setup |
| 374 | [GitHub OAuth Impl](phase-17-auth/374-github-oauth-impl.md) | github, oauth, implement, flow |
| 375 | [Google OAuth Config](phase-17-auth/375-google-oauth-config.md) | google, oauth, config, setup |
| 376 | [Google OAuth Impl](phase-17-auth/376-google-oauth-impl.md) | google, oauth, implement, flow |
| 377 | [Magic Link Types](phase-17-auth/377-magic-link-types.md) | magic, link, type, struct |
| 378 | [Magic Link Email](phase-17-auth/378-magic-link-email.md) | magic, link, email, send |
| 379 | [Magic Link Verification](phase-17-auth/379-magic-link-verify.md) | magic, link, verify, validate |
| 380 | [Okta Integration Config](phase-17-auth/380-okta-config.md) | okta, config, setup, integration |
| 381 | [Okta Integration Impl](phase-17-auth/381-okta-impl.md) | okta, implement, flow, sso |
| 382 | [Auth Middleware](phase-17-auth/382-auth-middleware.md) | auth, middleware, protect, guard |
| 383 | [Auth API Endpoints](phase-17-auth/383-auth-api.md) | auth, api, endpoint, login |
| 384 | [Auth UI Integration](phase-17-auth/384-auth-ui.md) | auth, ui, login, form |
| 385 | [Auth Error Handling](phase-17-auth/385-auth-errors.md) | auth, error, handle, message |
| 386 | [Auth Rate Limiting](phase-17-auth/386-auth-rate-limit.md) | auth, rate, limit, brute-force |
| 387 | [Auth Audit Logging](phase-17-auth/387-auth-audit.md) | auth, audit, log, track |
| 388 | [Auth Session Cleanup](phase-17-auth/388-auth-cleanup.md) | session, cleanup, expire, prune |
| 389 | [Multi-Tenant Auth](phase-17-auth/389-multi-tenant.md) | multi, tenant, org, team |
| 390 | [Auth Tests](phase-17-auth/390-auth-tests.md) | test, auth, integration, e2e |

## Phase 18: Feature Flags System (391-410)
| Spec | File | Keywords |
|------|------|----------|
| 391 | [Feature Flags Core Types](phase-18-flags/391-flags-core-types.md) | flag, core, type, struct |
| 392 | [Flag Definition Format](phase-18-flags/392-flag-definition.md) | flag, definition, format, schema |
| 393 | [Flag Storage](phase-18-flags/393-flag-storage.md) | flag, storage, database, persist |
| 394 | [Flag Evaluation Engine](phase-18-flags/394-flag-evaluation.md) | flag, evaluate, engine, check |
| 395 | [Flag Context Types](phase-18-flags/395-flag-context.md) | flag, context, user, environment |
| 396 | [Percentage Rollout](phase-18-flags/396-percentage-rollout.md) | percentage, rollout, gradual |
| 397 | [User Targeting](phase-18-flags/397-user-targeting.md) | user, target, specific, id |
| 398 | [Group Targeting](phase-18-flags/398-group-targeting.md) | group, target, segment, cohort |
| 399 | [A/B Testing Support](phase-18-flags/399-ab-testing.md) | ab, test, experiment, variant |
| 400 | [Flag Override API](phase-18-flags/400-flag-override.md) | flag, override, api, force |
| 401 | [Flag Admin UI](phase-18-flags/401-flag-admin-ui.md) | flag, admin, ui, manage |
| 402 | [Flag SDK Rust](phase-18-flags/402-flag-sdk-rust.md) | flag, sdk, rust, client |
| 403 | [Flag SDK TypeScript](phase-18-flags/403-flag-sdk-ts.md) | flag, sdk, typescript, client |
| 404 | [Flag Sync Mechanism](phase-18-flags/404-flag-sync.md) | flag, sync, refresh, update |
| 405 | [Flag Caching](phase-18-flags/405-flag-caching.md) | flag, cache, local, ttl |
| 406 | [Flag Analytics](phase-18-flags/406-flag-analytics.md) | flag, analytics, track, usage |
| 407 | [Flag Audit Trail](phase-18-flags/407-flag-audit.md) | flag, audit, trail, history |
| 408 | [Flag Deprecation](phase-18-flags/408-flag-deprecation.md) | flag, deprecate, remove, cleanup |
| 409 | [Feature Flag API](phase-18-flags/409-flag-api.md) | flag, api, endpoint, rest |
| 410 | [Feature Flag Tests](phase-18-flags/410-flag-tests.md) | test, flag, integration, e2e |

## Phase 19: Analytics System (411-430)
| Spec | File | Keywords |
|------|------|----------|
| 411 | [Analytics Event Types](phase-19-analytics/411-event-types.md) | analytics, event, type, struct |
| 412 | [Analytics Event Schema](phase-19-analytics/412-event-schema.md) | analytics, schema, event, format |
| 413 | [Event Capture API](phase-19-analytics/413-event-capture.md) | event, capture, api, track |
| 414 | [Event Batching](phase-19-analytics/414-event-batching.md) | event, batch, queue, flush |
| 415 | [Event Persistence](phase-19-analytics/415-event-persistence.md) | event, persist, store, database |
| 416 | [Event Aggregation](phase-19-analytics/416-event-aggregation.md) | event, aggregate, sum, count |
| 417 | [User Identification](phase-19-analytics/417-user-identification.md) | user, identify, id, anonymous |
| 418 | [Session Tracking](phase-19-analytics/418-session-tracking.md) | session, track, start, end |
| 419 | [Page View Tracking](phase-19-analytics/419-pageview-tracking.md) | page, view, track, navigation |
| 420 | [Action Tracking](phase-19-analytics/420-action-tracking.md) | action, track, click, event |
| 421 | [Error Tracking](phase-19-analytics/421-error-tracking.md) | error, track, exception, capture |
| 422 | [Performance Tracking](phase-19-analytics/422-performance-tracking.md) | performance, track, timing, metric |
| 423 | [Analytics Query API](phase-19-analytics/423-analytics-query.md) | analytics, query, api, filter |
| 424 | [Analytics Export](phase-19-analytics/424-analytics-export.md) | analytics, export, csv, json |
| 425 | [Privacy Compliance](phase-19-analytics/425-privacy-compliance.md) | privacy, gdpr, consent, anonymize |
| 426 | [Data Retention Policies](phase-19-analytics/426-data-retention.md) | data, retention, policy, expire |
| 427 | [Analytics Dashboard Data](phase-19-analytics/427-dashboard-data.md) | analytics, dashboard, data, chart |
| 428 | [Real-time Analytics](phase-19-analytics/428-realtime-analytics.md) | realtime, analytics, live, stream |
| 429 | [Analytics Webhooks](phase-19-analytics/429-analytics-webhooks.md) | analytics, webhook, notify, event |
| 430 | [Analytics Tests](phase-19-analytics/430-analytics-tests.md) | test, analytics, integration |

## Phase 20: Audit System (431-450)
| Spec | File | Keywords |
|------|------|----------|
| 431 | [Audit Event Types](phase-20-audit/431-audit-event-types.md) | audit, event, type, struct |
| 432 | [Audit Event Schema](phase-20-audit/432-audit-schema.md) | audit, schema, event, format |
| 433 | [Audit Capture Middleware](phase-20-audit/433-audit-capture.md) | audit, capture, middleware, auto |
| 434 | [Audit Persistence](phase-20-audit/434-audit-persistence.md) | audit, persist, store, append |
| 435 | [Audit Query API](phase-20-audit/435-audit-query.md) | audit, query, api, search |
| 436 | [Audit Retention](phase-20-audit/436-audit-retention.md) | audit, retention, policy, expire |
| 437 | [Audit Export](phase-20-audit/437-audit-export.md) | audit, export, csv, json |
| 438 | [Audit Search](phase-20-audit/438-audit-search.md) | audit, search, filter, find |
| 439 | [Audit Filtering](phase-20-audit/439-audit-filtering.md) | audit, filter, type, user |
| 440 | [Audit Timeline View](phase-20-audit/440-audit-timeline.md) | audit, timeline, view, chronological |
| 441 | [Audit User Activity](phase-20-audit/441-audit-user-activity.md) | audit, user, activity, track |
| 442 | [Audit System Events](phase-20-audit/442-audit-system-events.md) | audit, system, event, internal |
| 443 | [Audit Security Events](phase-20-audit/443-audit-security.md) | audit, security, event, alert |
| 444 | [Audit Compliance Reports](phase-20-audit/444-audit-compliance.md) | audit, compliance, report, soc2 |
| 445 | [Audit Alerting](phase-20-audit/445-audit-alerting.md) | audit, alert, notify, trigger |
| 446 | [Audit Immutability](phase-20-audit/446-audit-immutability.md) | audit, immutable, tamper, proof |
| 447 | [Audit Archival](phase-20-audit/447-audit-archival.md) | audit, archive, cold, storage |
| 448 | [Audit GDPR Compliance](phase-20-audit/448-audit-gdpr.md) | audit, gdpr, privacy, delete |
| 449 | [Audit API](phase-20-audit/449-audit-api.md) | audit, api, endpoint, rest |
| 450 | [Audit Tests](phase-20-audit/450-audit-tests.md) | test, audit, integration |

## Phase 21: VCS Integration - jj-first (451-471)

**Note:** jj (Jujutsu) is the primary VCS for agentic coding. It offers superior conflict handling, concurrent edits, and easy undo/redo. Git compatibility is provided for remotes.

**BUILD ORDER:** Run jj specs (471a-471i) FIRST, then git fallback specs (451-470).

| Spec | File | Keywords |
|------|------|----------|
| 471a | [VCS Crate Setup](phase-21-vcs/471a-vcs-crate-setup.md) | vcs, jj, crate, setup |
| 471b | [jj Repository](phase-21-vcs/471b-jj-repository.md) | jj, repo, detect, init |
| 471c | [jj Status](phase-21-vcs/471c-jj-status.md) | jj, status, diff, changes |
| 471d | [jj Commit](phase-21-vcs/471d-jj-commit.md) | jj, commit, describe, new |
| 471e | [jj Conflicts](phase-21-vcs/471e-jj-conflicts.md) | jj, conflict, resolve, merge |
| 471f | [jj Undo/Redo](phase-21-vcs/471f-jj-undo.md) | jj, undo, redo, operation |
| 471g | [jj Branches](phase-21-vcs/471g-jj-branches.md) | jj, branch, create, track |
| 471h | [Git Compatibility](phase-21-vcs/471h-git-compat.md) | jj, git, push, fetch, compat |
| 471i | [VCS Tests](phase-21-vcs/471i-vcs-tests.md) | vcs, jj, test, integration |
| 471j | [jj Conflict Introspection](phase-21-vcs/471j-jj-conflict-introspection.md) | jj, conflict, structured, api |
| 451 | [Git Core Types](phase-21-git/451-git-core-types.md) | git, core, type, struct |
| 452 | [Git Repository Detection](phase-21-git/452-git-detect.md) | git, detect, repository, find |
| 453 | [Git Status Operations](phase-21-git/453-git-status.md) | git, status, changed, staged |
| 454 | [Git Diff Operations](phase-21-git/454-git-diff.md) | git, diff, compare, change |
| 455 | [Git Commit Operations](phase-21-git/455-git-commit.md) | git, commit, message, create |
| 456 | [Git Branch Operations](phase-21-git/456-git-branch.md) | git, branch, create, switch |
| 457 | [Git Push Operations](phase-21-git/457-git-push.md) | git, push, remote, upstream |
| 458 | [Git Pull Operations](phase-21-git/458-git-pull.md) | git, pull, fetch, merge |
| 459 | [Git Merge Handling](phase-21-git/459-git-merge.md) | git, merge, branch, combine |
| 460 | [Git Conflict Detection](phase-21-git/460-git-conflict.md) | git, conflict, detect, resolve |
| 461 | [Git History Parsing](phase-21-git/461-git-history.md) | git, history, log, parse |
| 462 | [Git Blame Integration](phase-21-git/462-git-blame.md) | git, blame, author, line |
| 463 | [Git Hooks Support](phase-21-git/463-git-hooks.md) | git, hook, pre-commit, post |
| 464 | [Git Credential Handling](phase-21-git/464-git-credentials.md) | git, credential, auth, store |
| 465 | [Git SSH Key Management](phase-21-git/465-git-ssh.md) | git, ssh, key, manage |
| 466 | [Git Remote Management](phase-21-git/466-git-remote.md) | git, remote, add, origin |
| 467 | [Git Worktree Support](phase-21-git/467-git-worktree.md) | git, worktree, multiple, checkout |
| 468 | [Git LFS Handling](phase-21-git/468-git-lfs.md) | git, lfs, large, file |
| 469 | [Git API](phase-21-git/469-git-api.md) | git, api, endpoint, operations |
| 470 | [Git Tests](phase-21-git/470-git-tests.md) | test, git, integration |

## Phase 22: Testing Infrastructure (471-490)
| Spec | File | Keywords |
|------|------|----------|
| 471 | [Test Harness Setup](phase-22-testing/471-test-harness.md) | test, harness, setup, framework |
| 472 | [Unit Test Patterns](phase-22-testing/472-unit-patterns.md) | unit, test, pattern, best-practice |
| 473 | [Integration Test Patterns](phase-22-testing/473-integration-patterns.md) | integration, test, pattern, e2e |
| 474 | [Property Testing Setup](phase-22-testing/474-property-testing.md) | property, test, quickcheck, proptest |
| 475 | [Snapshot Testing](phase-22-testing/475-snapshot-testing.md) | snapshot, test, insta, compare |
| 476 | [Mock Backends](phase-22-testing/476-mock-backends.md) | mock, backend, fake, stub |
| 477 | [Mock File System](phase-22-testing/477-mock-filesystem.md) | mock, filesystem, fake, temp |
| 478 | [Mock Network](phase-22-testing/478-mock-network.md) | mock, network, http, intercept |
| 479 | [Test Fixtures](phase-22-testing/479-test-fixtures.md) | fixture, test, data, setup |
| 480 | [Test Data Generators](phase-22-testing/480-test-generators.md) | generate, test, data, fake |
| 481 | [Test Coverage Setup](phase-22-testing/481-test-coverage.md) | coverage, test, lcov, report |
| 482 | [Test Reporting](phase-22-testing/482-test-reporting.md) | report, test, junit, html |
| 483 | [E2E Test Framework](phase-22-testing/483-e2e-framework.md) | e2e, test, framework, playwright |
| 484 | [E2E Test Scenarios](phase-22-testing/484-e2e-scenarios.md) | e2e, test, scenario, case |
| 485 | [Visual Regression Tests](phase-22-testing/485-visual-regression.md) | visual, regression, screenshot |
| 486 | [Performance Benchmarks](phase-22-testing/486-benchmarks.md) | benchmark, performance, criterion |
| 487 | [Load Testing Setup](phase-22-testing/487-load-testing.md) | load, test, stress, k6 |
| 488 | [Test CI Integration](phase-22-testing/488-test-ci.md) | test, ci, github-actions, run |
| 489 | [Flaky Test Handling](phase-22-testing/489-flaky-tests.md) | flaky, test, retry, quarantine |
| 490 | [Test Documentation](phase-22-testing/490-test-docs.md) | test, documentation, guide |

## Phase 23: Build & Distribution (491-510)
| Spec | File | Keywords |
|------|------|----------|
| 491 | [Build System Overview](phase-23-build/491-build-overview.md) | build, system, overview, pipeline |
| 492 | [Rust Build Config](phase-23-build/492-rust-build.md) | rust, build, cargo, config |
| 493 | [TypeScript Build Config](phase-23-build/493-ts-build.md) | typescript, build, vite, config |
| 494 | [Electron Packaging](phase-23-build/494-electron-packaging.md) | electron, package, bundle, app |
| 495 | [macOS DMG Creation](phase-23-build/495-macos-dmg.md) | macos, dmg, create, package |
| 496 | [macOS Code Signing](phase-23-build/496-macos-signing.md) | macos, sign, code, certificate |
| 497 | [macOS Notarization](phase-23-build/497-macos-notarize.md) | macos, notarize, apple, submit |
| 498 | [Windows Installer](phase-23-build/498-windows-installer.md) | windows, installer, nsis, msi |
| 499 | [Windows Code Signing](phase-23-build/499-windows-signing.md) | windows, sign, code, certificate |
| 500 | [Linux AppImage](phase-23-build/500-linux-appimage.md) | linux, appimage, portable |
| 501 | [Linux Debian Package](phase-23-build/501-linux-deb.md) | linux, debian, deb, package |
| 502 | [Auto-Update Server](phase-23-build/502-auto-update-server.md) | update, server, host, release |
| 503 | [Auto-Update Client](phase-23-build/503-auto-update-client.md) | update, client, check, download |
| 504 | [Version Management](phase-23-build/504-version-management.md) | version, semver, bump, manage |
| 505 | [Release Tagging](phase-23-build/505-release-tagging.md) | release, tag, git, version |
| 506 | [Changelog Generation](phase-23-build/506-changelog.md) | changelog, generate, commit, history |
| 507 | [Release Notes](phase-23-build/507-release-notes.md) | release, notes, write, publish |
| 508 | [Distribution CDN](phase-23-build/508-distribution-cdn.md) | distribution, cdn, host, download |
| 509 | [Download Page](phase-23-build/509-download-page.md) | download, page, web, links |
| 510 | [Build Tests](phase-23-build/510-build-tests.md) | test, build, integration, ci |

## Phase 24: Documentation System (511-525)
| Spec | File | Keywords |
|------|------|----------|
| 511 | [Documentation Structure](phase-24-docs/511-doc-structure.md) | documentation, structure, organize |
| 512 | [API Documentation](phase-24-docs/512-api-docs.md) | api, documentation, generate, openapi |
| 513 | [CLI Documentation](phase-24-docs/513-cli-docs.md) | cli, documentation, generate, help |
| 514 | [User Guide Outline](phase-24-docs/514-user-guide.md) | user, guide, outline, tutorial |
| 515 | [Quick Start Guide](phase-24-docs/515-quick-start.md) | quick, start, guide, getting-started |
| 516 | [Configuration Reference](phase-24-docs/516-config-reference.md) | config, reference, documentation |
| 517 | [Spec Writing Guide](phase-24-docs/517-spec-writing.md) | spec, writing, guide, howto |
| 518 | [Troubleshooting Guide](phase-24-docs/518-troubleshooting.md) | troubleshooting, guide, problem, fix |
| 519 | [FAQ Content](phase-24-docs/519-faq.md) | faq, question, answer, common |
| 520 | [Video Tutorial Scripts](phase-24-docs/520-video-scripts.md) | video, tutorial, script, screencast |
| 521 | [Example Projects](phase-24-docs/521-example-projects.md) | example, project, sample, demo |
| 522 | [Migration Guides](phase-24-docs/522-migration-guides.md) | migration, guide, upgrade, version |
| 523 | [Contribution Guidelines](phase-24-docs/523-contributing.md) | contributing, guide, contribution |
| 524 | [Security Policy](phase-24-docs/524-security-policy.md) | security, policy, vulnerability, report |
| 525 | [Documentation Site](phase-24-docs/525-doc-site.md) | documentation, site, build, deploy |

## Phase 25: Advanced Features (526-550)
| Spec | File | Keywords |
|------|------|----------|
| 526 | [K8s Pod Types](phase-25-advanced/526-k8s-pod-types.md) | k8s, pod, type, struct |
| 527 | [K8s Pod Management](phase-25-advanced/527-k8s-pod-management.md) | k8s, pod, manage, create |
| 528 | [SPIFFE Identity Integration](phase-25-advanced/528-spiffe-identity.md) | spiffe, identity, integrate |
| 529 | [SPIFFE Certificate Handling](phase-25-advanced/529-spiffe-cert.md) | spiffe, certificate, handle |
| 530 | [WireGuard Tunnel Setup](phase-25-advanced/530-wireguard-tunnel.md) | wireguard, tunnel, setup, vpn |
| 531 | [DERP Relay Integration](phase-25-advanced/531-derp-relay.md) | derp, relay, integrate, tailscale |
| 532 | [eBPF Audit Sidecar Types](phase-25-advanced/532-ebpf-types.md) | ebpf, audit, sidecar, type |
| 533 | [eBPF Event Capture](phase-25-advanced/533-ebpf-capture.md) | ebpf, event, capture, syscall |
| 534 | [Remote Execution API](phase-25-advanced/534-remote-exec-api.md) | remote, execution, api, endpoint |
| 535 | [Remote Session Management](phase-25-advanced/535-remote-session.md) | remote, session, manage, connect |
| 536 | [Container Image Building](phase-25-advanced/536-container-image.md) | container, image, build, docker |
| 537 | [Nix Integration](phase-25-advanced/537-nix-integration.md) | nix, integration, reproducible |
| 538 | [Self-Update Mechanism](phase-25-advanced/538-self-update.md) | self, update, mechanism, upgrade |
| 539 | [Plugin System Types](phase-25-advanced/539-plugin-types.md) | plugin, type, struct, interface |
| 540 | [Plugin Loading](phase-25-advanced/540-plugin-loading.md) | plugin, load, dynamic, register |
| 541 | [Plugin API](phase-25-advanced/541-plugin-api.md) | plugin, api, hook, extension |
| 542 | [Multi-Agent Communication](phase-25-advanced/542-multi-agent-comm.md) | multi, agent, communication, sync |
| 543 | [Multi-Agent Coordination](phase-25-advanced/543-multi-agent-coord.md) | multi, agent, coordinate, orchestrate |
| 544 | [Knowledge Base System](phase-25-advanced/544-knowledge-base.md) | knowledge, base, system, learn |
| 545 | [Self-Improvement Loop](phase-25-advanced/545-self-improve.md) | self, improvement, loop, evolve |
| 546 | [Spec Auto-Modification](phase-25-advanced/546-spec-auto-mod.md) | spec, auto, modify, update |
| 547 | [Metric-Based Rollback](phase-25-advanced/547-metric-rollback.md) | metric, rollback, auto, revert |
| 548 | [Experiment Framework](phase-25-advanced/548-experiment-framework.md) | experiment, framework, ab, test |
| 549 | [Advanced Monitoring](phase-25-advanced/549-advanced-monitoring.md) | advanced, monitoring, observe |
| 550 | [Advanced Feature Tests](phase-25-advanced/550-advanced-tests.md) | test, advanced, integration |

---

## Quick Reference

### By Component
- **CLI**: 076-095
- **Loop**: 096-115
- **Forge**: 136-160, 256-275
- **Electron**: 161-185
- **UI**: 186-315
- **Server**: 316-340
- **Database**: 341-365
- **Auth**: 366-390

### By Technology
- **Rust**: 011-075, 316-390, 411-470
- **TypeScript/Svelte**: 186-315
- **Electron**: 161-185
- **SQLite/PostgreSQL**: 341-365

### Mission-Critical Paths
1. **MVP**: 001-010 → 031-050 → 056-060 → 096-115 → 161-175 → 186-215
2. **Forge**: 136-160 → 256-275
3. **Full Stack**: MVP → 316-390 → 341-365


## Phase 26: Hotfix - Critical UI (561-570)

**PRIORITY: P0 - These specs fix the broken UI that was neglected**

| Spec | File | Keywords |
|------|------|----------|
| 561 | [App Shell Sidebar](phase-26-hotfix/561-app-shell-sidebar.md) | layout, sidebar, shell, navigation |
| 562 | [Wire Layout to Routes](phase-26-hotfix/562-wire-layout-to-routes.md) | layout, routes, wire |
| 563 | [Dashboard Page](phase-26-hotfix/563-dashboard-page.md) | dashboard, home, stats |
| 564 | [Missions Route](phase-26-hotfix/564-missions-route.md) | missions, route, history |
| 565 | [Specs Route](phase-26-hotfix/565-specs-route.md) | specs, browser, route |
| 566 | [Forge Route](phase-26-hotfix/566-forge-route.md) | forge, brainstorm, route |
| 567 | [Settings Route](phase-26-hotfix/567-settings-route.md) | settings, config, route |
| 551 | [Wire Main Layout](phase-09-ui-foundation/551-wire-main-layout.md) | layout, critical |
| 552 | [Electron Dev Fixes](phase-08-electron/552-electron-dev-fixes.md) | electron, dev, fixes |

## Phase 27: Ralph TUI Integration (568-573)

**PRIORITY: P1 - UX polish inspired by Ralph TUI**

| Spec | File | Keywords |
|------|------|----------|
| 568 | [Application Shell](phase-27-ralph-integration/568-application-shell.md) | shell, integration, layout, crypto |
| 569 | [Conversational Spec Creation](phase-27-ralph-integration/569-conversational-spec-creation.md) | chat, prd, interview, spec |
| 570 | [Quickstart Onboarding](phase-27-ralph-integration/570-quickstart-onboarding.md) | init, setup, doctor, onboard |
| 571 | [TUI Loop Runner](phase-27-ralph-integration/571-tui-loop-runner.md) | tui, ratatui, loop, terminal |
| 572 | [Plugin System](phase-27-ralph-integration/572-plugin-system.md) | plugins, agents, trackers, templates |
| 573 | [Simple Config Mode](phase-27-ralph-integration/573-simple-config-mode.md) | config, simple, progressive |
| 574 | [Electron Dev Server](phase-26-hotfix/574-electron-dev-server.md) | electron, dev, port, crypto |

## Phase 28: Forge LLM Integration (575-586)

**PRIORITY: P0 - Wire real LLM calls to the Forge Think Tank**

| Spec | File | Keywords |
|------|------|----------|
| 575 | [LLM Provider Trait](phase-28-forge-llm/575-llm-provider-trait.md) | trait, llm, provider, types |
| 576 | [Anthropic Provider](phase-28-forge-llm/576-anthropic-provider.md) | claude, anthropic, stream, sse |
| 577 | [Forge Orchestrator](phase-28-forge-llm/577-forge-orchestrator.md) | orchestrator, round, deliberate |
| 578 | [Server Integration](phase-28-forge-llm/578-server-integration.md) | server, api, websocket, routes |
| 579 | [Consensus Summary](phase-28-forge-llm/579-consensus-summary.md) | summary, output, human, decision |
| 580 | [Beadifier](phase-28-forge-llm/580-beadifier.md) | beads, tasks, atomic, decompose |
| 581 | [NAPI-RS Forge Bindings](phase-28-forge-llm/581-napi-forge-bindings.md) | napi, electron, bindings, rust |
| 582 | [Participant Model Config](phase-28-forge-llm/582-participant-model-config.md) | participant, model, config, multi |
| 583 | [Agent Role Prompts](phase-28-forge-llm/583-agent-role-prompts.md) | agent, role, prompt, AGENTS.md |
| 584 | [Divergent Deliberation](phase-28-forge-llm/584-divergent-deliberation.md) | diverge, disagree, refine, converge |
| 585 | [Multi-Provider Orchestrator](phase-28-forge-llm/585-multi-provider-orchestrator.md) | openai, ollama, multi, factory |
| 586 | [Frontend-Backend Wire](phase-28-forge-llm/586-frontend-backend-wire.md) | frontend, ipc, stream, wire |