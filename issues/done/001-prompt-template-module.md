# Issue 1: Pure prompt_template module with unit tests

**Type**: AFK

## What to build

Create a new pure Rust module `src-tauri/src/prompt_template.rs` that exposes a deterministic, zero-copy placeholder substitution function for post-processing prompt templates. The module provides an `EMPTY_CUSTOM_WORDS` constant (`"(none provided)"`), a borrowed `PromptContext<'a>` struct holding `output` and `custom_words` fields, and a `render(template, ctx) -> String` function that performs dumb `str::replace` for each placeholder. The module has no I/O and no Tauri dependency, is registered as `mod prompt_template;` in `lib.rs`, and follows the `#[cfg(test)] mod tests` pattern established in `src-tauri/src/audio_toolkit/text.rs`.

## Acceptance criteria

- [ ] `src-tauri/src/prompt_template.rs` exists with `pub const EMPTY_CUSTOM_WORDS: &str = "(none provided)";`
- [ ] `pub struct PromptContext<'a> { pub output: &'a str, pub custom_words: &'a str }` is defined
- [ ] `pub fn render(template: &str, ctx: &PromptContext) -> String` substitutes `${output}` and `${custom_words}` via `str::replace`
- [ ] Module is declared in `lib.rs` as `mod prompt_template;`
- [ ] No I/O, no Tauri dependency, no `unwrap()` in the module
- [ ] Unit tests cover: both placeholders substituted; only `${output}` present (custom_words literal untouched); only `${custom_words}` present (output literal untouched); `custom_words` equal to `EMPTY_CUSTOM_WORDS` renders literally; neither placeholder present (returned unchanged); multiple occurrences of the same placeholder all substituted; special characters (`$`, `{`, `}`) outside a placeholder cause no accidental substitution
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo test` passes (new module tests green)
- [ ] `cargo build` passes

## Blocked by

- None - can start immediately

## User stories covered

- 2. As a Handy user, I want to reference my custom words in my prompt via a `${custom_words}` placeholder, so that I control where and how the list appears in my instructions.
- 4. As a Handy user with no custom words configured, I want the prompt to still make sense, so that the LLM isn't confused by an empty "Custom Words:" section.
- 6. As a Handy user, I want to be able to omit `${custom_words}` from my custom prompt entirely, so that the list is only sent when I explicitly want it.
- 14. As a Handy user, I want the placeholder substitution to be deterministic and pure, so that prompt rendering is predictable and testable.
