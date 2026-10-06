## What changed

<!-- Describe the user-visible behaviour that changed. -->

## Why

<!-- Link the issue this closes, if any, with `Closes #123`. -->

## How it was tested

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`

Hardware testing, because most of this project cannot be covered by unit tests:

- [ ] External monitor over DDC/CI
- [ ] Internal laptop panel
- [ ] Diagnostic panel, with a permission deliberately missing

Operating system and desktop environment used:

<!-- For example: Pop!_OS 24.04, COSMIC, Wayland, AMD GPU -->

## Checklist

- [ ] The engine (`tauri-brightness-core`) still has no Tauri dependency
- [ ] New public items are documented, because both crates deny `missing_docs`
- [ ] `api-iife.js` was regenerated with `npm run build` and committed, if the
      bindings changed
- [ ] `CHANGELOG.md` and a file under `.changes/` were updated, if the change
      should be released
