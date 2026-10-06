import { invoke } from '@tauri-apps/api/core';

// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
/**
 * Control display brightness through DDC/CI and the Linux kernel backlight.
 *
 * The bindings talk to the `brightness` plugin registered by
 * `tauri_plugin_brightness::init()`. Every function rejects with a readable
 * string when the backend fails, for example when an external monitor rejects
 * the DDC/CI request because the user is not in the `i2c` group.
 *
 * @module
 */
/**
 * Returns every display whose brightness can be controlled.
 *
 * The list is recomputed on every call, so monitors plugged in after start-up
 * appear without restarting the application.
 *
 * @example
 * ```typescript
 * import { listMonitors } from 'tauri-plugin-brightness-api'
 *
 * const monitors = await listMonitors()
 * for (const monitor of monitors) {
 *   console.log(`${monitor.name} is at ${monitor.brightness}/${monitor.max}`)
 * }
 * ```
 *
 * @returns A promise resolving to the detected displays, which is empty when
 * none was found. Call {@linkcode diagnose} to learn why.
 */
async function listMonitors() {
    return await invoke('plugin:brightness|list_monitors');
}
/**
 * Reads the brightness of one display.
 *
 * @example
 * ```typescript
 * import { getBrightness } from 'tauri-plugin-brightness-api'
 *
 * const { brightness, max, brightnessPercent } = await getBrightness('ddc-0')
 * console.log(`${brightnessPercent}% (${brightness} of ${max})`)
 * ```
 *
 * @param id A display identifier returned by {@linkcode listMonitors}.
 * @returns A promise resolving to the current value, on the display's own scale
 * plus a comparable percentage.
 */
async function getBrightness(id) {
    return await invoke('plugin:brightness|get_brightness', {
        id
    });
}
/**
 * Sets the brightness of one display.
 *
 * @example
 * ```typescript
 * import { setBrightness } from 'tauri-plugin-brightness-api'
 *
 * await setBrightness('ddc-0', 50)
 * ```
 *
 * @param id A display identifier returned by {@linkcode listMonitors}.
 * @param value The target brightness. DDC/CI displays that report a maximum of
 * `100` expect a percentage; other displays expect a value on their own scale.
 * @returns A promise that resolves once the display acknowledged the change.
 */
async function setBrightness(id, value) {
    return await invoke('plugin:brightness|set_brightness', { id, value });
}
/**
 * Probes the system and explains why a display may not have been detected.
 *
 * @example
 * ```typescript
 * import { diagnose } from 'tauri-plugin-brightness-api'
 *
 * const report = await diagnose()
 * if (!report.ddcutilAvailable) {
 *   console.warn(report.suggestions.join('\n'))
 * }
 * ```
 *
 * @returns A promise resolving to the environment checks and the suggested
 * fixes. Never rejects.
 */
async function diagnose() {
    return await invoke('plugin:brightness|diagnose');
}

export { diagnose, getBrightness, listMonitors, setBrightness };
