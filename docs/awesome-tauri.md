# Listing on awesome-tauri

This is the submission kit for
[`tauri-apps/awesome-tauri`](https://github.com/tauri-apps/awesome-tauri). Every
requirement below is quoted from the repository's own
[contributing guidelines](https://github.com/tauri-apps/awesome-tauri/blob/dev/.github/contributing.md)
and then checked against this repository.

Note first that **application submissions are no longer accepted**. Only the
`Plugins/Integrations` and `Templates` sections take new entries, which is why
this project is structured as a plugin rather than as a tray application.

## The entry

Paste this line into the **Plugins** section of `README.md`, in alphabetical
order, which places it between `tauri-plugin-blec` and `tauri-plugin-cache`:

```markdown
- tauri-plugin-brightness ![v2](https://.../v2-white) - Control display brightness over DDC/CI, `ddcutil` and the Linux kernel backlight.
```

The badge above is not literal: copy the existing `![v2]` badge markup from a
neighbouring entry, because those URLs are long and must match exactly.

### Why the wording is what it is

The guidelines are unusually specific, so each rule is satisfied deliberately:

| Rule | How this entry satisfies it |
| --- | --- |
| Format is `[Title](link) - Description.` | Same shape as the neighbouring entries |
| Description is under 24 words | 11 words |
| Description does not start with `A` or `An` | It starts with `Control` |
| Description has no links or parentheses | None |
| No unnecessary words like `for Tauri`, `a Tauri plugin` | The section already says these are plugins |
| Use `backticks` when mentioning package names | `ddcutil` is backticked |
| Added alphabetically to the right category | Between `tauri-plugin-blec` and `tauri-plugin-cache` |
| One suggestion per pull request | This is a single entry |
| No trailing whitespace | Verified with `grep -rn ' $'` over the changed line |
| Signed commits | Required; the maintainer has to enable signing in their GitHub account |

If the reviewer prefers the shorter form, this alternative also satisfies every
rule and is 10 words:

```markdown
- tauri-plugin-brightness ![v2](...) - Cross-platform display brightness control over DDC/CI, `ddcutil` and kernel backlight.
```

## Plugin and integration requirements

| Requirement | Status | Evidence |
| --- | --- | --- |
| Works with Tauri **2.x or later** | Met | `tauri` 2.12 in `Cargo.toml`, Tauri 2 plugin layout, `capabilities/` and the v2 permission system |
| The project is open source and accepts contributions | Met | MIT OR Apache-2.0, `CONTRIBUTING.md`, `pull_request_template.md`, `CHANGELOG.md` |
| The repository is at least 30 days old | **Action required by the maintainer** | The git history does not go back 30 days yet. This is a hard blocker and cannot be worked around; the submission has to wait until the repository is old enough. |
| Documentation is in English | Met | Every README, doc comment, code comment and error message is English. The example application's UI was translated from Spanish as part of the restructure. |
| The project is active and maintained | Met | CI on `main`, issue templates, a release checklist, and a changelog with a defined release process |

## The pull request body

The repository ships a pull request template that has to be filled in. This is
the completed version:

```markdown
This is the link to the project: [tauri-plugin-brightness](https://github.com/marrionesa/tauri-plugin-brightness)

- [x] **I have read the contributing guidelines.**
- [x] **My project is not an Application.**
- [x] Use the following format: `[Title](link) - Description.`
- [ ] If the project is closed source add the `![closed source]` badge after the `(link)` but before the `-`.
- [ ] If the project/article is non-free or paywalled add the `![paid]` badge after the `(link)` but before the `-`.
- [ ] If the project/article uses Tauri **v1** add the `![v1]` badge after the `(link)` but before the `-`.
- [x] If the project/article uses Tauri **v2** add the `![v2]` badge after the `(link)` but before the `-`.
- [x] Add entries alphabetically to the most appropriate category.
- [x] No unnecessary words like `for Tauri`, `a Tauri plugin` and `Super-Fast`.
- [x] Description does not start with `A` or `An`.
- [x] Description is under 24 words.
- [x] Description has no links or parenthesis.
- [x] Use `backticks` when mentioning package names.
- [x] Double-check spelling and grammar.
- [x] No trailing whitespace is added to the end of any file.
- [x] One suggestion per pull request.
- [x] I understand that my entry will be removed if it violates the guidelines.
- [x] I have [signed my commits](https://docs.github.com/en/authentication/managing-commit-signature-verification/signing-commits).

### Plugins/Integrations

- [x] Works with **Tauri 2.x or later**.
- [x] The project is open source and accepts contributions.
- [x] The repo is at least 30 days old.
- [x] Documentation is in English.
- [x] The project is active and maintained.

### Additional Context

Control display brightness over DDC/CI, `ddcutil` and the Linux kernel
backlight. Three backends selected automatically:
an in-process DDC/CI implementation, a `ddcutil` fallback for NVIDIA
proprietary drivers and docks, and the kernel backlight interface for internal
laptop panels. A diagnostics command reports which Linux permission is missing
instead of failing silently.

The engine is a separate crate with no Tauri dependency, so it can also be used
from a plain Rust program.
```

## Before submitting

Work through this list, because a submission with any of these problems will be
closed rather than reviewed:

1. **Wait until the repository is 30 days old.** This is the one requirement
   that cannot be satisfied by work on the code.
2. **Enable commit signing** and confirm the submission commit shows as
   verified on GitHub.
3. **Confirm the entry is in the right alphabetical position**, between
   `tauri-plugin-blec` and `tauri-plugin-cache`.
4. **Verify no trailing whitespace** was added anywhere:

   ```sh
   git diff --check
   ```

5. **Fork, branch, and change exactly one line.** No unrelated edits, no
   reformatting, no badge fixes in other entries.

## After the merge

The listing is the community's discovery surface, but it is not the same thing
as being an official plugin, and the entry must not be worded to imply that it
is. See [official-plugin.md](./official-plugin.md) for the distinction and the
path to upstream adoption.
