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

| Module                     | Status     | Notes                                        |
| -------------------------- | ---------- | -------------------------------------------- |
| Audio Recording Manager    | active     | Stable, cpal + device enumeration.           |
| Model Manager              | active     | Download, discovery, size tracking.          |
| Transcription Manager      | active     | Whisper + Parakeet inference.                |
| History Manager            | active     | SQLite via rusqlite.                         |
| Transcription Coordinator  | active     | Single-thread lifecycle serialisation.       |
| Audio Toolkit              | active     | VAD, recording, text filtering.              |
| Shortcut Handling          | active     | Being rewritten (Globe key support in prog). |
| Clipboard / Input          | active     | Platform-specific paste methods.             |
| LLM Client                 | active     | OpenAI-compatible post-processing.           |
| Prompt Template            | active     | Pure std-only placeholder substitution (`prompt_template.rs`). Issue 001. |
| Settings                   | active     | ~990 lines, needs refactoring.               |
| Overlay                    | active     | NSPanel (macOS), GTK layer shell (Linux).    |
| Tray                       | active     | State-driven icons + context menu.           |
| Portable Mode              | active     | Marker file + Data/ directory.               |
| Frontend (React)           | active     | Settings UI, onboarding, model selector.     |

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

## Session Delta — Issue 001 (Pure prompt_template module)

**Structural changes the next agent will encounter:**
- New file `src-tauri/src/prompt_template.rs` exposing:
  - `pub const EMPTY_CUSTOM_WORDS: &str = "(none provided)";`
  - `pub struct PromptContext<'a> { pub output: &'a str, pub custom_words: &'a str }`
  - `pub fn render(template: &str, ctx: &PromptContext) -> String` — chained `str::replace` for `${output}` then `${custom_words}`.
- `mod prompt_template;` added to `lib.rs` (between `portable` and `settings`). The module is **private** (`mod`, not `pub mod`); access its items via `crate::prompt_template::render` / `crate::prompt_template::PromptContext` / `crate::prompt_template::EMPTY_CUSTOM_WORDS`.

**Gotchas / time-savers:**
- `render` substitutes `${output}` first, then `${custom_words}` — if a substituted value itself contains the other placeholder, it will cascade. Issue 002 should be aware of this if it ever feeds user-controlled text into `output`.
- The module has no external deps, so it can be unit-tested in isolation with `rustc --test` even when the full `cargo build` is blocked by the `libx11-dev` issue above.
- 7 unit tests cover every acceptance scenario and all pass (verified via `rustc --test`).
- The `#[cfg(test)] mod tests { use super::*; }` pattern mirrors `src-tauri/src/audio_toolkit/text.rs`.

**Next issue (002) — what it needs from this one:**
- Issue 002 wires `prompt_template::render` into `post_process_transcription` (`actions.rs`), updates the default "Improve Transcriptions" prompt text, and routes `${custom_words}` through `build_system_prompt` for the structured-output / Apple Intelligence paths. It needs exactly `render`, `PromptContext`, and `EMPTY_CUSTOM_WORDS` — all now available via `crate::prompt_template::`. No `bindings.ts` regen needed (no schema change).

## Next Unimplemented Issue

**002** — Wire `${custom_words}` into the post-processing pipeline (`issues/002-wire-custom-words-into-post-process-pipeline.md`). Blocked-by Issue 001 is now satisfied.
