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

## Next Unimplemented Issue

TBD — awaiting `issues/` directory.
