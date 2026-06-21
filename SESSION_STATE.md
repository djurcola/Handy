# SESSION_STATE.md

Day 0 baseline for autonomous agent handoff.

## Project Phase

**Active development** (v0.8.3) with a **feature freeze** in effect. Bug fixes are prioritised over new features.

## Technology Stack

- **Framework:** Tauri 2.x (Rust backend + React 18 frontend)
- **Frontend:** React 18, TypeScript 5.6, Vite 6, Tailwind CSS 4, Zustand 5, i18next
- **Backend:** Rust (stable), `transcribe-rs` (whisper.cpp + ONNX), `cpal`, `rdev`, `enigo`, `rubato`, `rodio`, `tokio`
- **Build:** tauri-specta (type generation), cargo (Rust), bun (JS), NSIS (Windows installer)
- **CI:** GitHub Actions (build-test, code-quality, playwright, nix-check, release workflows)

## Build, Test, and Lint Commands

```bash
bun install                          # Install JS dependencies
bun run tauri dev                    # Dev mode (full app)
bun run tauri build                  # Production build
bun run dev                          # Frontend-only (Vite dev server on :1420)
bun run build                        # Build frontend (tsc + vite)
bun run lint                         # ESLint (frontend)
bun run lint:fix                     # ESLint with auto-fix
bun run format                       # Prettier + cargo fmt
bun run format:check                 # Check formatting only
bun run test:playwright              # E2E tests
bun run test:playwright:ui           # E2E tests with UI
bun run check:translations           # Validate i18n keys

# Rust-only
cargo fmt                            # Format Rust code
cargo clippy                         # Lint Rust code
cargo test                           # Rust tests (minimal coverage)
```

## Module Status

All modules are **implemented** (active maintenance).

| Module                    | Status | Notes                                                                     |
| ------------------------- | ------ | ------------------------------------------------------------------------- |
| Audio Recording Manager   | active | Stable, cpal + device enumeration.                                        |
| Model Manager             | active | Download, discovery, size tracking.                                       |
| Transcription Manager     | active | Whisper + Parakeet inference.                                             |
| History Manager           | active | SQLite via rusqlite.                                                      |
| Transcription Coordinator | active | Single-thread lifecycle serialisation.                                    |
| Audio Toolkit             | active | VAD, recording, text filtering.                                           |
| Shortcut Handling         | active | Being rewritten (Globe key support in prog).                              |
| Clipboard / Input         | active | Platform-specific paste methods.                                          |
| LLM Client                | active | OpenAI-compatible post-processing.                                        |
| Prompt Template           | active | Pure std-only placeholder substitution (`prompt_template.rs`). Issue 001. |
| Settings                  | active | ~990 lines, needs refactoring.                                            |
| Overlay                   | active | NSPanel (macOS), GTK layer shell (Linux).                                 |
| Tray                      | active | State-driven icons + context menu.                                        |
| Portable Mode             | active | Marker file + Data/ directory.                                            |
| Frontend (React)          | active | Settings UI, onboarding, model selector.                                  |

## Key Configuration

- **Dev server:** `http://localhost:1420`
- **App identifier:** `com.pais.handy`
- **Data directory:** Platform-dependent (see `docs/architecture.md` or `portable.rs`)
- **Model files:** `{app_data}/models/` for Whisper `.bin` and Parakeet dirs; `src-tauri/resources/models/silero_vad_v4.onnx` for dev
- **Env vars:**
  - `RUST_LOG` — log filter override (console)
  - `HANDY_NO_GTK_LAYER_SHELL=1` — disable GTK overlay on Linux
  - `WEBKIT_DISABLE_DMABUF_RENDERER=1` — workaround Linux GPU issues
  - `CMAKE_POLICY_VERSION_MINIMUM=3.5` — macOS cmake compat

## Key Documentation

- `AGENTS.md` — agent guidance and repo conventions
- `CONTEXT.md` — domain glossary
- `docs/architecture.md` — system design and module boundaries
- `docs/adr/` — architecture decision records (currently empty)
- `CONTRIBUTING.md` — contributor workflow and code style
- `BUILD.md` — platform-specific build instructions
- `README.md` — public documentation

## Open Questions / Blockers

- Settings refactoring is flagged on the roadmap but not yet started.
- `bindings.ts` regeneration is not automated.
- No Rust-level test suite exists; only E2E via Playwright.
- Whisper crash investigation ongoing (config-dependent, hard to reproduce).
- **Environment blocker (this machine):** `cargo clippy` / `cargo build` / `cargo test` fail because `libx11-dev` is not installed (the `x11` crate, a transitive dep of `rdev` for X11 keyboard shortcuts, cannot find `x11.pc` via pkg-config). No passwordless sudo to install it. This is pre-existing and unrelated to the `prompt_template` module. Pure std-only modules can still be verified standalone with `rustc --test <file>`. `cargo fmt --check` works. No `clippy.toml` or crate-level `#![deny]` lints exist.

## Session Delta — Issue 002 (Wire ${custom_words} into the post-processing pipeline)

**Structural changes the next agent will encounter:**

- `build_system_prompt` in `actions.rs` now takes a second arg `custom_words: &str` and delegates to `prompt_template::render` (strip `${output}` → trim → render `${custom_words}`). The single call site is `actions.rs:168`.
- `post_process_transcription` in `actions.rs` builds `custom_words_str` (newline-joined `settings.custom_words`, or `EMPTY_CUSTOM_WORDS` sentinel when empty) and a `PromptContext` before branching. Legacy path calls `prompt_template::render` directly with both placeholders; structured-output path goes through `build_system_prompt`. Apple Intelligence is covered automatically via `build_system_prompt`.
- Default "Improve Transcriptions" prompt in `settings.rs` (`default_post_process_prompts`) now has a 6th instruction step ("Review output against the Custom Words and ensure that any obvious mis-transcriptions are replaced") and a `Custom Words:\n${custom_words}` section before `Transcript:\n${output}`. Only affects fresh installs (serde `default =` on the field); no migration of existing users' customized prompts.

**Gotchas / time-savers:**

- The `${custom_words}` placeholder now functions end-to-end across every `PostProcessProvider` variant (OpenAI, OpenRouter, Anthropic, Apple Intelligence, custom) and both prompt modes (legacy + structured-output). Issue 003 documents it in the UI tip.
- `cargo fmt --check` passes; `cargo clippy`/`cargo build` still blocked by the pre-existing libx11-dev issue (see Open Questions above). Verified logic via 7 `prompt_template` unit tests (`rustc --test`) + 5 standalone integration checks simulating both paths against the new default prompt.
- The frontend commands (`bun run lint`, `bun run format:check`, `bun run build`) are NOT blocked by the libx11-dev issue — they should work normally for Issue 003's i18n-only changes.
- `apply_custom_words` local fuzzy matcher in `transcription.rs` is unchanged (still runs pre-LLM for non-Whisper models). `llm_client.rs` is unchanged (already accepts arbitrary prompt strings). No new settings, no new Tauri command, no `bindings.ts` regen.

**Next issue (003) — what it needs from this one:**

- Issue 003 updates two English i18n keys in `src/i18n/locales/en/translation.json` to document both `${output}` and `${custom_words}` placeholders in the post-processing prompt UI tip. The placeholder now actually functions (Issue 002 satisfied). It should NOT change `promptInstructionsPlaceholder` or any non-English locale.

## Session Delta — Issue 003 (Document ${custom_words} placeholder in the prompt UI tip)

**Structural changes the next agent will encounter:**

- `src/i18n/locales/en/translation.json`: two English keys updated under `settings.postProcessing.prompts` — `selectedPrompt.description` and `promptTip` now document both `${output}` and `${custom_words}`. `promptInstructionsPlaceholder` unchanged. No non-English locale touched.

**Gotchas / time-savers:**

- `bun` was not installed on this machine; installed via `curl -fsSL https://bun.sh/install | bash` (binary at `~/.bun/bin/bun`). `bun install` then succeeded. Future agents on this host may need `export PATH="$HOME/.bun/bin:$PATH"` to run frontend commands.
- `bun run lint` passes; `bun run build` passes. `bun run format:check` reports 6 pre-existing formatting warnings (`AGENTS.md`, `CONTEXT.md`, `docs/architecture.md`, `docs/prd/PRD-custom_words_in_post_process_prompt.md`, `SESSION_STATE.md`, `src/components/ui/Dropdown.tsx`) — none are the edited translation file. These pre-date Issue 003 (confirmed by stashing the change and re-running). Not fixed to keep the commit scoped.
- The UI surfaces these keys via `i18nKey` in `PostProcessingSettings.tsx:254`, `:320`, `:387` (Trans component), so English users see the updated tip without any component changes.
- This commit also folded in Issue 002's leftover Phase 5/6 artifacts (issue file move to `done/`, SESSION_STATE.md delta) that the prior session left uncommitted.

**Next issue — what it needs from this one:**

- None. All three issues (001, 002, 003) in the `issues/` directory are now in `issues/done/`. The custom-words-in-post-process-prompt feature is complete end-to-end (module → pipeline wiring → user-facing docs).

## Next Unimplemented Issue

**None** — all issues in `issues/` are complete (001, 002, 003 all in `issues/done/`).
