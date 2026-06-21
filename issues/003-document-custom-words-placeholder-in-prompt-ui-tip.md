# Issue 3: Document ${custom_words} placeholder in the prompt UI tip (i18n)

**Type**: AFK

## What to build

Update the English i18n strings so the post-processing prompt UI tip documents both the existing `${output}` placeholder and the new `${custom_words}` placeholder. This is the user-facing discovery mechanism for the feature: users editing their prompt learn that `${custom_words}` injects their custom words list. Only the English locale is updated; other locales are left for translators per the project's translation contribution guidelines. No other UI surface changes.

## Acceptance criteria

- [ ] `settings.postProcessing.prompts.selectedPrompt.description` in `src/i18n/locales/en/translation.json` reads: `"Use ${output} to reference the captured transcript and ${custom_words} to include your custom words list."`
- [ ] `settings.postProcessing.prompts.promptTip` in `src/i18n/locales/en/translation.json` reads: `"Tip: Use <code>${output}</code> to insert the transcribed text and <code>${custom_words}</code> to include your custom words list."`
- [ ] `promptInstructionsPlaceholder` is left unchanged (example text, not documentation)
- [ ] No other locales are modified
- [ ] No hardcoded JSX strings added (i18n enforced by ESLint)
- [ ] `bun run lint` passes
- [ ] `bun run format:check` passes
- [ ] `bun run build` passes
- [ ] Settings UI displays the updated tip text in English, documenting both placeholders

## Blocked by

- Issue 002 (Wire ${custom_words} into the post-processing pipeline) — the placeholder must actually function before documenting it to users

## User stories covered

- 7. As a Handy user editing my prompt, I want the UI tip to document both `${output}` and `${custom_words}`, so that I discover the placeholder without reading docs.
