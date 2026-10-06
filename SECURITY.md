# Security Policy

## Supported versions

Only the latest published version of each crate and of the npm package receives
security fixes, because the API is young and back-porting would delay the fix.

| Package | Supported |
| --- | --- |
| `tauri-brightness-core` | Latest only |
| `tauri-plugin-brightness` | Latest only |
| `tauri-plugin-brightness-api` | Latest only |

## Reporting a vulnerability

Report vulnerabilities privately through
[GitHub Security Advisories](https://github.com/marrionesa/tauri-plugin-brightness/security/advisories/new),
which keeps the report, the discussion and the fix private until a release is
out.

Please do not open a public issue for a vulnerability.

Include, as far as you can:

- The package and version affected.
- What an attacker can achieve, and what access they need to start.
- Steps to reproduce, or a proof of concept.
- Any suggested fix.

You can expect an acknowledgement within a few days. This is a spare-time
project, so please allow reasonable time for a fix before disclosing publicly.

## Scope

The interesting attack surface in this project is narrow and worth stating
plainly, because most of it is not exploitable in the usual sense.

**In scope:**

- **Helper command invocation.** The engine spawns `ddcutil`, `dbus-send` and
  `brightnessctl`. None of them is invoked through a shell, and no argument is
  ever built from a value that a webview supplies; display identifiers are
  parsed into a backend enum plus a numeric index or sysfs device name before
  use. A way to break that would be a real finding.
- **Path handling in the backlight backend.** Device names come from
  `/sys/class/backlight` enumeration, but a crafted identifier passed to
  `set_brightness` is worth probing for path traversal. See the fixed
  vulnerability below for what that looked like.
- **Permission definitions.** A way to call a command that the application's
  capability does not grant.
- **The npm package and the IIFE bundle.** Anything that would make the guest
  bindings call a command other than the documented one.

**Out of scope:**

- **Requiring privileges.** Controlling an external monitor needs access to
  `/dev/i2c-*`, which usually means the operator added themselves to the `i2c`
  group or is `root`. That is by design, documented, and not a vulnerability in
  this project.
- **A local attacker changing the screen brightness.** Anyone who can already
  run code as the same user can do that directly.
- **The displays or the kernel doing the wrong thing.** A monitor that ignores
  DDC/CI writes is a hardware limitation, not a security issue.
- **Denial of service through the diagnostic command.** It runs `ddcutil detect`
  and reads `/proc` and `/sys`, all as the current user, and it is documented as
  slow.

## Design notes that reduce risk

- No command is invoked through a shell, so there is no shell injection surface.
- Rust code forbids `unsafe` in the engine (`#![deny(unsafe_code)]`).
- Display identifiers are parsed into a typed enum before any dispatch, so a
  string from the webview never reaches a filesystem path or a command argument
  unvalidated.
- The plugin grants nothing by default beyond the standard set, and the
  capability system decides which windows may call which command.

## Fixed vulnerabilities

### Path traversal through a backlight display identifier

**Affected:** `tauri-brightness-core` 0.1.0 and `tauri-plugin-brightness` 0.1.0.
**Fixed in:** 0.1.1.

An identifier such as `backlight-../../etc/passwd` was accepted, and the
backlight backend joined its locator onto `/sys/class/backlight` without
validation. That produced a path outside the sysfs directory, so a caller that
could pass an arbitrary identifier could have made the plugin read, or attempt
to write, an arbitrary file as the user running the application.

The identifier normally originates from `list_monitors`, so exploitation
required an application to forward a value it did not get from the plugin,
which is why this is a bounded rather than a remote vulnerability. It was found
during a security review of this repository, not reported by anyone.

The fix validates the locator at parse time, restricting it to a kernel device
name, and validates it again where the path is built, because a `MonitorId` can
be constructed directly and bypass the parser. Two tests cover it: one feeds
nine traversal payloads to the parser, and one calls the path builder directly
to prove the second layer works on its own.

Thanks go to nobody but the review that found it; if you find a variant that
still escapes, please report it privately as described above.
