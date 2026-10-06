/** The mechanism a display is driven through. */
export type Backend = 'ddc' | 'ddcutil' | 'backlight';
/** A display whose brightness can be read and written. */
export interface MonitorInfo {
    /** Identifier to pass to {@linkcode getBrightness} and {@linkcode setBrightness}. */
    id: string;
    /** Human readable model name, or a generated fallback such as `Monitor 1`. */
    name: string;
    /** Current brightness, on a `0..=max` scale. */
    brightness: number;
    /** Highest value the display accepts. */
    max: number;
    /** Which mechanism drives this display. */
    backend: Backend;
}
/** The response of {@linkcode getBrightness}. */
export interface BrightnessPayload {
    /** The display the value belongs to. */
    id: string;
    /** The current brightness on the display's own scale. */
    brightness: number;
    /** The highest value the display accepts. */
    max: number;
    /** The current brightness as a percentage of `max`. */
    brightnessPercent: number;
}
/** The result of {@linkcode diagnose}. */
export interface Diagnostics {
    /** Whether the `i2c-dev` kernel module is loaded. */
    i2cDevLoaded: boolean;
    /** Whether `/sys/class/i2c-dev` exists. */
    i2cDevSysfs: boolean;
    /** The `/dev/i2c-*` device nodes that were found. */
    i2cDevices: string[];
    /** Whether the current user belongs to the `i2c` group. */
    inI2cGroup: boolean;
    /** Whether the current user belongs to the `video` group. */
    inVideoGroup: boolean;
    /** Whether `ddcutil` is on `PATH`. */
    ddcutilAvailable: boolean;
    /** Raw `ddcutil detect` output, useful when filing a bug report. */
    ddcutilDetectOutput: string;
    /** Number of displays the `ddc-hi` backend detected. */
    ddcHiCount: number;
    /** Number of displays the `ddcutil` backend detected. */
    ddcutilCount: number;
    /** Number of kernel backlight devices detected. */
    backlightCount: number;
    /** Whether an NVIDIA device was found. */
    hasNvidia: boolean;
    /** Actionable suggestions for fixing an empty detection. */
    suggestions: string[];
}
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
export declare function listMonitors(): Promise<MonitorInfo[]>;
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
export declare function getBrightness(id: string): Promise<BrightnessPayload>;
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
export declare function setBrightness(id: string, value: number): Promise<void>;
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
export declare function diagnose(): Promise<Diagnostics>;
