# CONTEXT.md

Domain glossary and terminology for Handy. **All agents must read this file before starting any work.**

## Core Concepts

| Term               | Definition                                                                                                                                |
| ------------------ | ----------------------------------------------------------------------------------------------------------------------------------------- |
| **Transcription**  | The conversion of spoken audio into text using a local ML model (Whisper or Parakeet).                                                    |
| **Post-processing** | Optional second pass over transcribed text using an LLM (OpenAI-compatible API) for grammar, formatting, or custom transformations.       |
| **VAD**            | Voice Activity Detection. Filtering silence from audio before transcription. Uses Silero ONNX model.                                      |
| **Recording**      | Capturing audio from the selected input device while the transcription shortcut is active.                                                |
| **Binding**        | A named keyboard shortcut that triggers a specific action (e.g., `transcribe`, `transcribe_with_post_process`).                           |
| **Push-to-talk**   | Recording mode where audio is captured only while the shortcut key is held down. Alternative to toggle mode.                              |
| **Toggle mode**    | Recording mode where one press starts recording, next press stops it.                                                                    |
| **Manager**        | A Rust struct holding shared state and business logic, stored in Tauri's managed state (e.g., `AudioRecordingManager`, `ModelManager`).  |
| **Coordinator**    | `TranscriptionCoordinator` — a single-threaded event loop that serialises all recording/transcription lifecycle events to avoid races.   |

## Model Terminology

| Term                  | Definition                                                                                                 |
| --------------------- | ---------------------------------------------------------------------------------------------------------- |
| **Whisper**           | OpenAI's speech-to-text model family. Available sizes: Small, Medium, Turbo, Large (quantised GGML bins).  |
| **Parakeet**          | NVIDIA's CPU-optimised speech recognition model family (ONNX runtime via `transcribe-rs`).                 |
| **GGML**              | A C library for running ML models on CPU/GPU with quantised weights. `.bin` files are GGML format models.  |
| **Silero VAD**        | Lightweight neural network for voice activity detection. Shipped as `silero_vad_v4.onnx` in resources.     |
| **transcribe-rs**     | Rust crate providing Whisper (via whisper.cpp) and Parakeet (via ONNX runtime) inference.                  |
| **Custom model**      | Any user-supplied Whisper GGML `.bin` placed in the `models/` directory, auto-discovered at startup.      |
| **Model provider**    | Local (Whisper/Parakeet) vs. remote LLM post-processing provider (OpenAI-compatible API).                  |

## Output Pipeline

| Term                 | Definition                                                                                                           |
| -------------------- | -------------------------------------------------------------------------------------------------------------------- |
| **Paste method**     | How transcribed text reaches the target app: Ctrl+V, Ctrl+Shift+V, Shift+Insert, or direct typing via enigo.        |
| **Clipboard handling** | Strategy for managing clipboard around paste: save/restore original content, or leave Handy text in clipboard.      |
| **Audio feedback**   | Optional audible cue (start/stop sounds) played via rodio when recording begins or ends.                             |
| **Auto-submit**      | Pressing Enter (or another key) automatically after pasting to confirm input in the target app.                      |
| **Custom words**     | User-defined word substitutions applied to transcription output (e.g., correcting domain-specific terms).             |
| **Apple Intelligence** | macOS-only feature using Apple's on-device LLM for post-processing (aarch64 only).                                 |

## Platform Concepts

| Term                    | Definition                                                                                             |
| ----------------------- | ------------------------------------------------------------------------------------------------------ |
| **Overlay**             | A small always-on-top window showing recording state. Uses GTK layer shell on Linux, NSPanel on macOS. |
| **Portable mode**       | Store all user data (settings, models, DB) in a `Data/` directory next to the executable instead of AppData. |
| **Single instance**     | Only one Handy process runs at a time. Second launch sends CLI args to the running instance and exits. |
| **Tray icon**           | System tray icon reflecting app state (Idle, Recording, Transcribing).                                 |
| **Signal handling**     | Unix signals (SIGUSR1, SIGUSR2) for remote control, mainly for Wayland WM integration.                 |
| **HANDY_NO_GTK_LAYER_SHELL** | Env var to disable gtk-layer-shell on Linux when the overlay causes focus-stealing issues.        |

## Settings Concepts

| Term              | Definition                                                                                     |
| ----------------- | ---------------------------------------------------------------------------------------------- |
| **Store plugin**  | `tauri-plugin-store` — persisted JSON settings file in the app data directory.                 |
| **Zustand store** | React state management library used by the frontend to mirror and mutate settings reactively.  |
| **bindings.ts**   | Auto-generated TypeScript types from Rust structs via `tauri-specta`. Do not edit manually.    |
| **i18n**          | Internationalisation via `react-i18next` / `i18next`. All JSX strings must use `t('key.path')`. |

## Architecture Patterns

| Term                        | Definition                                                                                                 |
| --------------------------- | ---------------------------------------------------------------------------------------------------------- |
| **Command-Event pattern**   | Frontend calls Rust via Tauri commands (RPC); Rust pushes updates to frontend via Tauri events.           |
| **Manager pattern**         | Core subsystems (Audio, Model, Transcription, History) encapsulated as arc-wrapped managers in Tauri state. |
| **TranscriptionCoordinator** | Owns a dedicated thread with an mpsc channel; serialises all recording/processing transitions.          |
| **FinishGuard**             | RAII drop guard that notifies the coordinator when the transcription pipeline finishes (even on panic).     |
