---
"tauri-plugin-brightness": patch:fix
"tauri-brightness-core": patch:fix
"tauri-plugin-brightness-api": patch:fix
---

Reject display identifiers that could escape the backlight directory.

An identifier such as `backlight-../../etc/passwd` was accepted and joined onto
`/sys/class/backlight`, producing a path outside it. Locators are now validated
both where they are parsed and where the path is built.
