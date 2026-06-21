# PRD: Custom Words in Post-Process Prompt

## Problem Statement

Handy already supports post-processing transcribed text through an OpenAI-compatible LLM (the "Improve Transcriptions" prompt cleans grammar, punctuation, number formatting, and filler words). It also maintains a user-configured list of **Custom Words** — domain-specific terms that should be preferred over likely mis-transcriptions.

However, the custom words list is currently never sent to the post-processing LLM. It is only used (a) as a Whisper `initial_prompt` bias and (b) as a local fuzzy string replacement for non-Whisper models. The local matcher can swap close phonetic matches but cannot add punctuation, fix capitalization contextually, or handle cases like `"daniel jurkolah"` → `"Daniel Jurcola"`.

As a result, a user recording "Hi my name is daniel jurkolah" with `Jurcola` in their custom words list gets the LLM's cleaned output, but the LLM has no knowledge that "jurkolah" should be "Jurcola" — so the name stays mis-transcribed. The user must manually post-edit, defeating the quality-of-life intent of post-processing.

## Solution

Introduce a new `${custom_words}` placeholder in the post-processing prompt template, alongside the existing `${output}` placeholder. When the post-processing pipeline runs, the user's custom words list is injected into the prompt wherever `${custom_words}` appears, so the LLM can review the transcription against the known domain terms and correct obvious mis-transcriptions while also fixing punctuation, capitalization, and formatting.

The default "Improve Transcriptions" prompt is updated to include a step instructing the model to review output against the custom words, plus a `Custom Words:\n${custom_words}` section. Users who edit their prompt can place `${custom_words}` wherever they like, or omit it entirely (opt-in via the template).

When the user has no custom words configured, `${custom_words}` resolves to the literal text `"(none provided)"` so prompts referencing the list still make sense to the model.

The existing local fuzzy matcher (`apply_custom_words`) continues to run before post-processing for non-Whisper models — belt-and-suspenders. No new settings toggle is introduced; `${custom_words}` is opt-in purely through prompt authoring.

## User Stories

1. As a Handy user, I want my custom words list to be sent to the post-processing LLM, so that the model can correct obvious mis-transcriptions of domain-specific terms.
2. As a Handy user, I want to reference my custom words in my prompt via a `${custom_words}` placeholder, so that I control where and how the list appears in my instructions.
3. As a Handy user who has not edited the default prompt, I want the default "Improve Transcriptions" prompt to already use my custom words, so that the feature works out of the box without prompt authoring.
4. As a Handy user with no custom words configured, I want the prompt to still make sense, so that the LLM isn't confused by an empty "Custom Words:" section.
5. As a Handy user, I want the local custom-words fuzzy matcher to keep running alongside LLM correction, so that obvious close matches are fixed even when the LLM misses them.
6. As a Handy user, I want to be able to omit `${custom_words}` from my custom prompt entirely, so that the list is only sent when I explicitly want it.
7. As a Handy user editing my prompt, I want the UI tip to document both `${output}` and `${custom_words}`, so that I discover the placeholder without reading docs.
8. As a Handy user, I want transcription to still work when post-processing is disabled, so that the custom-words placeholder change has no impact on the plain transcription flow.
9. As a Handy user who already customized my post-processing prompt, I want my prompt preserved, so that the upgrade doesn't overwrite my edits.
10. As a Handy user, I want `${custom_words}` to work with every supported provider (OpenAI, OpenRouter, Anthropic, Apple Intelligence, custom), so that I'm not locked to a specific backend.
11. As a Handy user, I want `${custom_words}` to work in both structured-output and legacy prompt modes, so that the behavior is consistent regardless of provider capabilities.
12. As a Handy user recording names, jargon, or product terms, I want the LLM to capitalize and punctuate corrected terms correctly (e.g. "daniel jurkolah" → "Daniel Jurcola"), so that the final pasted text is publish-ready.
13. As a Handy user, I want the feature to require no new settings UI, so that the settings surface stays lean.
14. As a Handy user, I want the placeholder substitution to be deterministic and pure, so that prompt rendering is predictable and testable.

## Implementation Decisions

- **New flat-file module: `src-tauri/src/prompt_template.rs`** — a pure Rust module exposing:
  - `pub const EMPTY_CUSTOM_WORDS: &str = "(none provided)";`
  - `pub struct PromptContext<'a> { pub output: &'a str, pub custom_words: &'a str }` (borrowed, zero-copy)
  - `pub fn render(template: &str, ctx: &PromptContext) -> String` — dumb `str::replace` for each placeholder; absent placeholders are naturally untouched.
  - A `#[cfg(test)] mod tests` block following the pattern in `audio_toolkit/text.rs`.
  - No I/O, no Tauri dependency — testable in isolation.
  - Registered as `mod prompt_template;` in `lib.rs`.
- **Integration via Option C (build_system_prompt wraps render):**
  - **Structured-output path**: the existing `build_system_prompt` function in `actions.rs` continues to strip `${output}` to empty string and trim. After stripping, it delegates to `prompt_template::render` to substitute `${custom_words}`. Apple Intelligence (which calls `build_system_prompt` at `actions.rs:147`) is automatically covered.
  - **Legacy path**: calls `prompt_template::render` directly with both placeholders substituted (`${output}` → transcription, `${custom_words}` → word list).
- **Caller formats the custom words string** — `post_process_transcription` in `actions.rs` builds the formatted string before constructing `PromptContext`:
  ```rust
  let cw = if settings.custom_words.is_empty() {
      prompt_template::EMPTY_CUSTOM_WORDS.to_string()
  } else {
      settings.custom_words.join("\n")
  };
  let ctx = PromptContext { output: &transcription, custom_words: &cw };
  ```
  The newline separator is chosen for LLM list readability (deliberately different from the comma-space format used for Whisper `initial_prompt`).
- **Modified: default prompt** — the default "Improve Transcriptions" prompt text gains a sixth instruction step and a `Custom Words` section before the `Transcript` section:
  ```
  Clean this transcript:
  1. Fix spelling, capitalization, and punctuation errors
  2. Convert number words to digits (twenty-five → 25, ten percent → 10%, five dollars → $5)
  3. Replace spoken punctuation with symbols (period → ., comma → ,, question mark → ?)
  4. Remove filler words (um, uh, like as filler)
  5. Keep the language in the original version (if it was french, keep it in french for example)
  6. Review output against the Custom Words and ensure that any obvious mis-transcriptions are replaced

  Preserve exact meaning and word order. Do not paraphrase or reorder content.

  Return only the cleaned transcript.

  Custom Words:
  ${custom_words}

  Transcript:
  ${output}
  ```
  Only affects new users / fresh installs; no migration of existing users' prompts (existing customized prompts are preserved verbatim). Users who never edited their default must manually reset or add `${custom_words}`. The updated UI tip is the discovery mechanism.
- **Empty-list contract** — the caller sets `PromptContext.custom_words` to `EMPTY_CUSTOM_WORDS` when the list is empty. The renderer does not strip the placeholder or surrounding section.
- **Local matcher unchanged** — `apply_custom_words` continues to run pre-LLM for non-Whisper models per the "keep both" decision. No change to its threshold setting or behavior.
- **No new settings, no new Tauri command, no schema change** — `custom_words` is already a persisted `Vec<String>` on `AppSettings` and already has a `update_custom_words` command. The LLM client (`llm_client.rs`) needs no change; it already accepts arbitrary prompt strings.
- **Provider-agnostic** — works with all `PostProcessProvider` variants including Apple Intelligence (which receives the rendered system prompt via native APIs) and structured-output providers (where `${output}` is stripped from the system prompt and the transcript is sent as the user message, while `${custom_words}` stays in the rendered system prompt).
- **Frontend i18n** — two English translation strings are updated:
  - `settings.postProcessing.prompts.selectedPrompt.description`: `"Use ${output} to reference the captured transcript and ${custom_words} to include your custom words list."`
  - `settings.postProcessing.prompts.promptTip`: `"Tip: Use <code>${output}</code> to insert the transcribed text and <code>${custom_words}</code> to include your custom words list."`
  - `promptInstructionsPlaceholder` is left unchanged (example text, not documentation).
  - Other locales are left for translators per the project's translation contribution guidelines.
- **bindings.ts** — no Rust struct shape changes, so no regeneration needed.

## Testing Decisions

- **What makes a good test here** — test external behavior of the pure `prompt_template::render` function: given a template string and a `PromptContext`, assert the exact rendered output. No I/O, no mocks, no implementation-detail assertions.
- **Module under test**: `prompt_template` only. `build_system_prompt` in `actions.rs` is a thin wrapper whose trim behavior is trivial and out of scope.
- **Test cases to cover**:
  1. Template with both `${output}` and `${custom_words}` → both substituted correctly
  2. Template with only `${output}` → `${custom_words}` literal remains unchanged
  3. Template with only `${custom_words}` → `${output}` literal remains unchanged
  4. `custom_words` value equals `EMPTY_CUSTOM_WORDS` (`"(none provided)"`) → renders literally
  5. Template with neither placeholder → returned unchanged
  6. Multiple occurrences of the same placeholder → all substituted
  7. Template/transcript containing special characters (`$`, `{`, `}`) outside a placeholder → no accidental substitution
- **Prior art**: `src-tauri/src/audio_toolkit/text.rs` contains an extensive `#[cfg(test)] mod tests` block exercising `apply_custom_words` and `filter_transcription_output` as pure functions — same pattern to follow.
- **No integration tests** for the LLM call path (would require HTTP mocking; out of scope). No E2E/Playwright additions (the change has no new UI surface).

## Out of Scope

- Any new settings toggle for "send custom words to LLM" — opt-in is via prompt authoring only.
- Migration of existing users' customized prompts to the new default.
- Passing custom words to Apple Intelligence via any channel other than the existing system-prompt path.
- Changes to the local `apply_custom_words` fuzzy matcher or its threshold.
- Changes to `llm_client.rs` request/response structs.
- Changes to the custom-words settings UI (`CustomWords.tsx`).
- Translation of the updated i18n tip into non-English locales.
- New Tauri commands or `bindings.ts` regeneration.
- Integration tests with HTTP mocking for the LLM call.

## Further Notes

- The feature is intentionally minimal: a single new pure module plus three small edits (orchestration call sites, default prompt text, i18n strings). The risk surface is small because the custom-words data already flows through the app — only the prompt-rendering step changes.
- The "keep both" decision means a corrected term may be substituted locally before the LLM sees it. This is desirable: the LLM then receives already-corrected text and the custom-words list as a consistency check, reducing the chance of the LLM "fixing" a correctly-substituted term back to a wrong form.
- Existing users who want the feature must either reset their prompt to the new default or manually add `${custom_words}` to their current prompt. The updated UI tip is the discovery mechanism.
- This PRD does not introduce an ADR. The placeholder-substitution pattern is a minor extension of the existing `${output}` convention; if future placeholders proliferate, an ADR on the prompt-template contract would become warranted.
