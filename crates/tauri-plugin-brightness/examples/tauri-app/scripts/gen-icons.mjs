// Generate PNG tray/window icons from icons/icon.svg using sharp.
//
// Outputs:
//   icons/icon.png          512x512  (master + macOS)
//   icons/32x32.png          32x32
//   icons/128x128.png       128x128
//   icons/128x128@2x.png    256x256  (high-DPI)
//
// For Windows .ico, run the official Tauri CLI:
//   bunx @tauri-apps/cli icon src-tauri/icons/icon.png
// That command regenerates ALL formats (including .ico and .icns) from the
// 512x512 master PNG.
//
// Run:  node scripts/gen-icons.mjs     (or `bun run gen:icons`)

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";
import sharp from "sharp";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ICONS_DIR = resolve(__dirname, "..", "src-tauri", "icons");
const SRC_SVG = resolve(ICONS_DIR, "icon.svg");
const TRAY_SVG = resolve(ICONS_DIR, "tray-icon.svg");

const svgBuffer = readFileSync(SRC_SVG);
const traySvgBuffer = readFileSync(TRAY_SVG);

const targets = [
  { file: "icon.png", size: 512 },
  { file: "32x32.png", size: 32 },
  { file: "128x128.png", size: 128 },
  { file: "128x128@2x.png", size: 256 },
];

for (const { file, size } of targets) {
  const out = resolve(ICONS_DIR, file);
  await sharp(svgBuffer, { density: 384 })
    .resize(size, size, { fit: "contain", background: { r: 0, g: 0, b: 0, alpha: 0 } })
    .png({ compressionLevel: 9 })
    .toFile(out);
  console.log(`  wrote ${out} (${size}x${size})`);
}

await sharp(traySvgBuffer, { density: 384 })
  .resize(32, 32, { fit: "contain", background: { r: 0, g: 0, b: 0, alpha: 0 } })
  .png({ compressionLevel: 9 })
  .toFile(resolve(ICONS_DIR, "tray-32x32.png"));
console.log(`  wrote ${resolve(ICONS_DIR, "tray-32x32.png")} (32x32)`);

console.log("icons generated.");
