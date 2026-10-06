# Contributing

Thanks for helping improve `tauri-plugin-brightness`.

## Repository layout

```text
crates/tauri-brightness-core/                 Rust engine, no Tauri dependency
crates/tauri-plugin-brightness/               the Tauri 2 plugin
crates/tauri-plugin-brightness/guest-js/      TypeScript bindings
crates/tauri-plugin-brightness/examples/      tray application example
```

The engine and the plugin are separate crates on purpose: the engine can be used
from any Rust program, and the plugin only adds the Tauri glue and the
permission definitions.

## Local setup

The workspace needs Rust **1.90** or newer, because that is what `tauri` 2.12
requires.

```sh
cargo test --workspace
cargo clippy --workspace --all-targets
cargo fmt --check
```

To work on the JavaScript bindings, and to regenerate the IIFE bundle that
`build.rs` embeds:

```sh
cd crates/tauri-plugin-brightness
npm install
npm run build
```

Always commit the regenerated `api-iife.js`: `cargo publish` ships it, so a
stale bundle would reach users.

## Testing against real hardware

Hardware behaviour cannot be covered by unit tests, so the example application
doubles as the integration test:

```sh
cd crates/tauri-plugin-brightness/examples/tauri-app
npm install
npm run dev
```

Before opening a pull request that touches a backend, please check it against at
least one external monitor and, when possible, one internal laptop panel. Say
which desktop environment and display you used in the pull request.

## Development guidelines

- Keep the engine free of Tauri dependencies; that is what makes it publishable
  on its own.
- Keep every parser pure and unit tested. All of the riskiest code in this
  project is text parsing of `ddcutil` output.
- Normalise brightness to `0..=100` only at the edges. Inside the engine, keep
  the display's own scale and use `percent` and `raw_from_percent` to convert.
- Never assume DDC/CI is available: every path must degrade to something the
  diagnostic panel can explain.
- Document new public items. Both crates deny `missing_docs`.

## Commit quality

- Describe the user-visible behaviour that changed.
- Include screenshots for UI changes in the example application.
- Mention the operating system and desktop environment used for hardware tests.

## Security

Do not paste private system information into issues. Diagnostic output can
include display names, device paths and group membership.

## Pull requests

- Sign your commits.
- One logical change per pull request.
- Update `CHANGELOG.md` and add a file under `.changes/` when the change should
  be released.
