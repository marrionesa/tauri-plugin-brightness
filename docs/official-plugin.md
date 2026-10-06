# The path to being an official plugin

This document explains what "official Tauri plugin" means, what the trademark
policy allows a third party to claim, and what this project would have to do to
be adopted upstream. It exists so that nobody has to guess, and so that no
statement in this repository overstates the status of the plugin.

## What the Tauri trademark policy says

The [Trademark Guidelines](https://tauri.app/about/trademark/) are explicit:

> You may publish the code for plugins and templates using the appropriate
> naming conventions, but please mention that these works are not officially
> approved. Only such codebases that are managed by the organization within the
> GitHub organization `tauri-apps` are considered official.

Two consequences follow directly, and this repository respects both:

1. **Nobody can declare their own plugin official.** The status comes from the
   codebase being managed inside the `tauri-apps` organisation. It cannot be
   earned by following conventions, and it cannot be self-published.
2. **An external plugin must say that it is not approved.** That is why the root
   README, the plugin README and the landing page all carry the disclaimer, and
   why every document in this repository does the same.

The policy also requires a trademark legend, which is present in the plugin
README and the landing page footer:

> TAURI is a trademark of The Tauri Programme within the Commons Conservancy.
> <https://tauri.app>

The naming conventions themselves are permitted. The policy requires using them
correctly, not avoiding them:

- The crate is `tauri-plugin-brightness`, the prefix Tauri reserves for plugins.
- The npm package is `@tauri-apps/plugin-brightness`, the scope convention.
- Neither registers a domain containing the mark.

## What "adoptable" means here

An external plugin is one pull request away from being official if it matches
the shape, tooling and quality bar of the code inside
`tauri-apps/plugins-workspace`. That is what this repository targets, and it is
checkable rather than aspirational.

| Convention in `plugins-workspace` | Status here |
| --- | --- |
| Crate at `tauri-plugin-{name}` with the reserved prefix | Met |
| `links = "tauri-plugin-brightness"` in `Cargo.toml` | Met |
| `build.rs` calling `tauri_plugin::Builder::new(COMMANDS)` | Met |
| `permissions/default.toml` plus auto-generated `allow-*` and `deny-*` | Met |
| `[package.metadata.platforms.support]` for the docs support table | Met |
| `src/{lib,commands,desktop,mobile,error,models}.rs` split | Met |
| `desktop.rs` and `mobile.rs` behind mutually exclusive `cfg` | Met |
| `guest-js/index.ts` with typed bindings and a documented module | Met |
| `api-iife.js` global bundle, wired through `global_api_script_path` | Met |
| npm package with `type: module`, `exports`, `files`, peer on the API | Met |
| Extension trait exposing the Rust API on any `Manager` | Met |
| Version bumping both crates together, `.changes/` notes | Met |
| English documentation, `missing_docs` denied | Met |
| Example application under `examples/` | Met |

The differences that remain are deliberate and are documented here rather than
hidden:

| Difference | Why |
| --- | --- |
| The engine is a separate crate, `tauri-brightness-core` | Brightness control is useful outside a webview, and keeping Tauri out of the engine is what makes that possible. Upstream plugins are single crates, so this would have to be folded in or kept as a twin release during adoption. |
| `edition = "2021"` | `plugins-workspace` uses edition 2024 and Rust 1.90. The edition can be raised, but the workspace as a whole is not on edition 2024 yet. |
| No `android/` or `ios/` project | Mobile is declared `none` in `platforms.support`, because display brightness is a desktop concept. Upstream plugins that declare mobile support ship those projects. |

## Steps to request adoption

1. **Open a suggestion issue in
   [`tauri-apps/plugins-workspace`](https://github.com/tauri-apps/plugins-workspace)**
   before writing any code. The contributing guide requires a greenlit
   suggestion for a new feature, and this is a new plugin.

   Make the case in terms of the gap it fills: the Tauri ecosystem has window
   transparency and vibrancy plugins, but no brightness control, and brightness
   is the one desktop affordance that has no web API at all.

2. **Join the Tauri Discord and raise it there**, which the contributing guide
   asks for explicitly, so the maintainers can point out problems early.

3. **Expect the crate boundary question.** The most likely outcome is that
   `tauri-brightness-core` is either vendored into the plugin or published
   alongside it. Both are fine; the engine has no Tauri dependency, so nothing
   about it needs to change.

4. **Submit the pull request** with the repository's rules in mind:

   - Commits must be signed.
   - The `AI Tool Policy` applies: any AI-assisted code must be reviewed and
     tested by the submitter, and AI must not be used to answer review comments.
     Everything in this repository has been compiled and tested, and the
     reviewer should be told that in the pull request body.
   - A changeset has to be added under `.changes/`, following
     `.changes/readme.md`. A ready-to-use file is already in this repository at
     `.changes/initial-release.md`.

5. **Be patient.** The issue list is not paid support, and a new plugin is a
   long-term maintenance commitment that the maintainers have to weigh against
   everything else.

## What to do until then

Publishing under the naming conventions, with the disclaimer, is exactly what
the trademark policy permits, and it is what other community plugins do. The
practical path is:

1. Publish both crates and the npm package. (`RELEASE.md` in the example
   application has the order, which matters because the plugin depends on the
   engine.)
2. Get listed in `awesome-tauri`, which is the community's discovery surface.
   See [awesome-tauri.md](./awesome-tauri.md).
3. Only then raise adoption upstream, with real users and real bug reports
   behind the request.

## What must never appear

Any of these would breach the trademark policy:

- "official Tauri brightness plugin"
- "approved by the Tauri Programme"
- The Tauri logo in a position or size that suggests endorsement. The wordmarks
  in prose are fine; the logos are not, without contacting
  `trademark@tauri.app` for licensing terms.
- A domain name containing the mark.
- Publishing the plugin code as modified Tauri core. This plugin does not modify
  Tauri at all, so this does not arise, but it matters for forks.
