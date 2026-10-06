# Release checklist

Both crates and the npm package are versioned together, even when only one of
them changed, which is the convention used by the official Tauri plugins.

## Before release

1. Update the version in every place it appears:

   - `Cargo.toml` (`[workspace.dependencies]` does not pin it, the crates do)
   - `crates/tauri-brightness-core/Cargo.toml`
   - `crates/tauri-plugin-brightness/Cargo.toml`
   - `crates/tauri-plugin-brightness/package.json`
   - `crates/tauri-plugin-brightness/examples/tauri-app/package.json`
   - `crates/tauri-plugin-brightness/examples/tauri-app/src-tauri/tauri.conf.json`

   Keep the `version` requirement of `tauri-brightness-core` inside
   `crates/tauri-plugin-brightness/Cargo.toml` in step with the core version, or
   the published plugin would depend on an older engine.

2. Move the `[Unreleased]` section of `CHANGELOG.md` into a new version section.

3. Regenerate the JavaScript bindings, and commit the result, because
   `cargo publish` ships the IIFE bundle that `build.rs` embeds:

   ```sh
   cd crates/tauri-plugin-brightness
   npm install
   npm run build
   ```

4. Run the checks:

   ```sh
   cargo fmt --all --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```

5. Test the example application against real hardware:

   - Test DDC/CI with at least one external monitor.
   - Test the internal panel on a laptop, if one is available.
   - Test the diagnostic panel: temporarily move the user out of the `i2c`
     group, or run without the `i2c-dev` module, and confirm the panel reports
     the right fix.

## Publishing

The engine must be published first: the plugin depends on it, so crates.io will
reject the plugin until the core is indexed.

```sh
cargo publish -p tauri-brightness-core
# wait for the index to pick it up, then
cargo publish -p tauri-plugin-brightness
```

Then the bindings:

```sh
cd crates/tauri-plugin-brightness
npm publish
```

## After publishing

- Tag the release: `git tag vX.Y.Z && git push --tags`.
- Confirm the badges in the root `README.md` resolve.
- Confirm `docs.rs` built both crates, including the permission reference.

## Notes on publishing to crates.io

- The `path` key of `tauri-brightness-core` inside the plugin is what makes the
  local build work. Cargo removes it on publish, and the registry copy is used
  instead, so no change is needed at release time.
- Do not publish the example application: it sets `publish = false`.
- The plugin's `exclude` list keeps `examples/`, `guest-js/`, `node_modules/`
  and the JavaScript build configuration out of the crate archive. Check it with
  `cargo package --list -p tauri-plugin-brightness` before publishing.
