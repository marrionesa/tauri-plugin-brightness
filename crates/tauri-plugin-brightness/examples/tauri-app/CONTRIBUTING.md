# Contributing to the example application

Thanks for helping improve the tray application example. The plugin itself has
its own guide in the [repository root](../../../../CONTRIBUTING.md); this one is
about the application shell.

## Local setup

```sh
cd crates/tauri-plugin-brightness/examples/tauri-app
bun install
bun run dev
```

Use `src/index.html` for quick frontend-only checks. The mock mode does not
touch hardware.

## Development guidelines

- Keep the frontend dependency-free unless a feature clearly needs otherwise.
  The point of this example is that it can be read end to end.
- Keep brightness logic out of this application. Every hardware access goes
  through the plugin, so anything this app can do is something the plugin does.
  If a feature needs a new command, add it to the plugin.
- Keep hardware access errors actionable for normal desktop users.
- Do not assume DDC/CI is available; always keep the diagnostic path usable.
- Test tray behavior on the target desktop environment when touching tray code.

## Commit quality

- Describe the user-visible behavior changed.
- Include screenshots for UI changes when opening a pull request.
- Mention the OS and desktop environment used for tray testing.

## Security

Do not paste private system information into issues. Diagnostic output may
include display names, device paths and group membership.
