# Issue 2: Wire ${custom_words} into the post-processing pipeline

**Type**: AFK

## What to build

Integrate the `prompt_template::render` function into the post-processing pipeline so that the user's custom words list is injected into the post-processing LLM prompt wherever `${custom_words}` appears. The caller (`post_process_transcription` in `actions.rs`) formats the custom words string (newline-joined, or `EMPTY_CUSTOM_WORDS` when empty) and constructs a `PromptContext` before rendering. The structured-output path delegates to `render` after `build_system_prompt` strips `${output}` and trims, so Apple Intelligence and structured-output providers receive `${custom_words}` in the system prompt. The legacy path calls `render` directly with both placeholders substituted. The default "Improve Transcriptions" prompt text is updated to include a sixth instruction step and a `Custom Words:\n${custom_words}` section before the `Transcript:` section. The local `apply_custom_words` fuzzy matcher continues to run pre-LLM for non-Whisper models, unchanged. No new settings, no new Tauri command, no schema change, no `bindings.ts` regeneration.

## Acceptance criteria

- [ ] `post_process_transcription` in `actions.rs` builds the formatted custom words string: `settings.custom_words.join("\n")`, or `prompt_template::EMPTY_CUSTOM_WORDS` when the list is empty
- [ ] Legacy post-process path calls `prompt_template::render` with both `${output}` (transcription) and `${custom_words}` (formatted list) substituted
- [ ] Structured-output path: `build_system_prompt` strips `${output}` to empty string and trims, then delegates to `prompt_template::render` to substitute `${custom_words}`
- [ ] Apple Intelligence path (which calls `build_system_prompt`) automatically receives the rendered custom words in its system prompt
- [ ] Works with every `PostProcessProvider` variant (OpenAI, OpenRouter, Anthropic, Apple Intelligence, custom)
- [ ] Default "Improve Transcriptions" prompt text includes a sixth instruction step ("Review output against the Custom Words and ensure that any obvious mis-transcriptions are replaced") and a `Custom Words:\n${custom_words}` section before the `Transcript:\n${output}` section
- [ ] Default prompt change affects only new users / fresh installs; existing customized prompts are preserved verbatim (no migration)
- [ ] `apply_custom_words` local fuzzy matcher is unchanged (still runs pre-LLM for non-Whisper models)
- [ ] Transcription still works when post-processing is disabled (placeholder change has no impact on the plain transcription flow)
- [ ] `llm_client.rs` requires no change (already accepts arbitrary prompt strings)
- [ ] No new settings toggle, no new Tauri command, no `bindings.ts` regeneration
- [ ] `cargo fmt --check`, `cargo clippy`, and `cargo build` pass
- [ ] End-to-end demo: with `Jurcola` in custom words and the default prompt, recording "hi my name is daniel jurkolah" yields "Hi my name is Daniel Jurcola" from a post-processing provider

## Blocked by

- Issue 001 (Pure prompt_template module with unit tests) — the render function and `PromptContext`/`EMPTY_CUSTOM_WORDS` types must exist before integration

## User stories covered

- 1. As a Handy user, I want my custom words list to be sent to the post-processing LLM, so that the model can correct obvious mis-transcriptions of domain-specific terms.
- 3. As a Handy user who has not edited the default prompt, I want the default "Improve Transcriptions" prompt to already use my custom words, so that the feature works out of the box without prompt authoring.
- 5. As a Handy user, I want the local custom-words fuzzy matcher to keep running alongside LLM correction, so that obvious close matches are fixed even when the LLM misses them.
- 8. As a Handy user, I want transcription to still work when post-processing is disabled, so that the custom-words placeholder change has no impact on the plain transcription flow.
- 9. As a Handy user who already customized my post-processing prompt, I want my prompt preserved, so that the upgrade doesn't overwrite my edits.
- 10. As a Handy user, I want `${custom_words}` to work with every supported provider (OpenAI, OpenRouter, Anthropic, Apple Intelligence, custom), so that I'm not locked to a specific backend.
- 11. As a Handy user, I want `${custom_words}` to work in both structured-output and legacy prompt modes, so that the behavior is consistent regardless of provider capabilities.
- 12. As a Handy user recording names, jargon, or product terms, I want the LLM to capitalize and punctuate corrected terms correctly (e.g. "daniel jurkolah" → "Daniel Jurcola"), so that the final pasted text is publish-ready.
- 13. As a Handy user, I want the feature to require no new settings UI, so that the settings surface stays lean.
