# Architecture

## System Overview

Handy is a Tauri 2.x desktop application: the frontend is React + TypeScript (Vite), the backend is Rust. All speech processing happens locally.

```
┌─────────────────────────────────────────────────────────────────┐
│                        Tauri Runtime                            │
│                                                                 │
│  ┌─────────────────────────┐    ┌──────────────────────────┐   │
│  │   React Frontend (Vite) │    │   Rust Backend            │   │
│  │                         │    │                           │   │
│  │  Zustand Store          │◄───┤  Tauri Commands (RPC)    │   │
│  │  react-i18next          │    │  Tauri Events (push)     │──►│
│  │  tauri-specta bindings  │    │                           │   │
│  │                         │    │  Managers                 │   │
│  │  Components:            │    │   ├ AudioRecordingManager │   │
│  │   Settings              │    │   ├ ModelManager          │   │
│  │   Onboarding            │    │   ├ TranscriptionManager  │   │
│  │   Model Selector        │    │   └ HistoryManager        │   │
│  │   Overlay               │    │                           │   │
│  │   Update Checker        │    │  TranscriptionCoordinator │   │
│  │                         │    │  Audio Toolkit             │   │
│  └─────────────────────────┘    │  Shortcut Handling         │   │
│                                 │  Clipboard / Input          │   │
│                                 │  LLM Client                 │   │
│                                 └──────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

## Data Flow

```
1. User presses shortcut key
   └─► rdev / Global Shortcut Plugin / Unix Signal
       └─► TranscriptionCoordinator.send_input()

2. Coordinator transitions: Idle → Recording
   └─► AudioRecordingManager.start_recording()
       └─► Audio Toolkit (cpal → raw PCM)
           └─► Silero VAD (silence filtering)

3. User releases shortcut (or toggles off)
   └─► Coordinator transitions: Recording → Processing
       └─► AudioRecordingManager.stop_recording() → Vec<f32>

4. TranscriptionManager.transcribe(audio)
   └─► transcribe-rs (Whisper GGML / Parakeet ONNX)
       └─► raw text

5. Post-processing (optional):
   └─► LLM Client (OpenAI-compatible API) or Apple Intelligence
       └─► refined text

6. Output:
   └─► clipboard::paste_text() → system clipboard + paste keystroke
       └─► text appears in active application

7. Coordinator transitions: Processing → Idle
   └─► HistoryManager.save() (SQLite via rusqlite)
```

## Module Boundaries

### Backend (`src-tauri/src/`)

| Module                         | Responsibility                                                               |
| ------------------------------ | ---------------------------------------------------------------------------- |
| `lib.rs`                       | App bootstrap, Tauri setup, manager init, logging config, tray.              |
| `main.rs`                      | CLI arg parsing (`clap`), portable mode init, calls into `lib.rs`.           |
| `managers/audio.rs`            | Audio device enumeration, recording start/stop, device selection.            |
| `managers/model.rs`            | Model download, discovery, size calculation, active model tracking.          |
| `managers/transcription.rs`    | Whisper and Parakeet inference, model hot-swapping.                          |
| `managers/history.rs`          | SQLite-backed transcription history CRUD.                                    |
| `transcription_coordinator.rs` | Single-threaded event loop serialising all recording transitions.            |
| `actions.rs`                   | Shortcut action trait + implementations (transcribe, post-process).          |
| `audio_toolkit/`               | Low-level audio: cpal recording, Silero VAD, device helpers, text filtering. |
| `audio_feedback.rs`            | Play start/stop sounds via rodio.                                            |
| `clipboard.rs`                 | Clipboard save/paste/restore with platform-specific paste methods.           |
| `input.rs`                     | Enigo-based keystroke simulation; platform-specific virtual keys.            |
| `llm_client.rs`                | OpenAI-compatible chat completion client for post-processing.                |
| `settings.rs`                  | `AppSettings` struct with serde, tauri-plugin-store persistence.             |
| `shortcut/`                    | Global shortcut registration and handling abstraction.                       |
| `overlay.rs`                   | Recording overlay window (platform-specific: NSPanel, GTK layer shell).      |
| `tray.rs` / `tray_i18n.rs`     | System tray icon, context menu, state-driven icon changes.                   |
| `portable.rs`                  | Portable mode detection and data path resolution.                            |
| `signal_handle.rs`             | Unix signal handlers (`SIGUSR1`, `SIGUSR2`) for remote control.              |
| `cli.rs`                       | CLI argument definitions via `clap` derive.                                  |
| `commands/`                    | Tauri command handlers (frontend-facing RPC).                                |
| `utils.rs`                     | Platform detection helpers (Wayland, KDE, etc.).                             |

### Frontend (`src/`)

| Module                       | Responsibility                                         |
| ---------------------------- | ------------------------------------------------------ |
| `App.tsx`                    | Root component, onboarding flow, debug shortcut.       |
| `stores/settingsStore.ts`    | Zustand store mirroring all app settings reactively.   |
| `hooks/useSettings.ts`       | Convenience hook wrapping the Zustand store.           |
| `bindings.ts`                | Auto-generated (tauri-specta). DO NOT edit manually.   |
| `components/settings/`       | Settings UI panels (audio, shortcuts, advanced, etc.). |
| `components/onboarding/`     | First-run setup experience.                            |
| `components/model-selector/` | Model download/switch UI.                              |
| `components/overlay/`        | Recording overlay UI (second webview window).          |
| `components/update-checker/` | App update notification.                               |
| `i18n/`                      | i18next setup + locale files (`locales/en/…`, etc.).   |

## Key Decisions

This project does not yet have an ADR registry. As ADRs are created, they should be listed below:

> No ADRs have been recorded yet. When adding one, create `docs/adr/NNNN-title.md` and add a link here.

Key architectural decisions that are implicit in the codebase (candidates for future ADRs):

1. **Single-threaded transcription coordinator** — All recording start/stop/processing transitions are serialised through one mpsc channel to eliminate race conditions between keyboard shortcuts, signals, and async pipelines. See `transcription_coordinator.rs`.
2. **Manager pattern with Arc-wrapped state** — Each subsystem is a separate struct stored in Tauri's managed state as `Arc<T>`. See `managers/*.rs`.
3. **tauri-specta for type generation** — Frontend TypeScript types are auto-generated from Rust structs. `bindings.ts` must never be hand-edited.
4. **i18n enforcement via ESLint** — `eslint-plugin-i18next` rejects hardcoded JSX strings. Any user-facing text must use `t('key.path')`.
5. **Feature freeze policy** — The project is under a feature freeze; bug fixes are prioritised. New features require community support via GitHub Discussions.
6. **Portable mode** — A marker file next to the executable opts into storing all data alongside the binary. See `portable.rs`.

## Known Weak Areas / Technical Debt

- **Settings system bloat** — The README explicitly calls out "settings refactoring" as needed. `settings.rs` is ~990 lines with a large flat `AppSettings` struct.
- **Tauri command organisation** — `commands/` is noted as needing cleanup and better abstractions.
- **Whisper model crashes** — Configuration-dependent crashes on Windows/Linux remain an open issue.
- **Wayland support** — Overlay focuses-stealing, shortcuts need WM manual config, limited integration.
- **Testing coverage** — Only one Playwright E2E test file (`tests/app.spec.ts`); no Rust unit/integration tests in `src-tauri/` (only `dev-dependencies = { tempfile = "3" }`).
- **`bindings.ts` regeneration** — Not automated on CI; must be regenerated when Rust structs change.
