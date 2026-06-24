import { convertFileSrc } from "@tauri-apps/api/core";
import { board } from "./board.svelte";

export type AssetKind = "image" | "video" | "audio" | "file";

// One markdown image/link reference, embed (`![]()`) or plain (`[]()`).
export const ASSET_RE = /(?:!?)\[[^\]]*\]\([^)]+\)/g;

const IMAGE_EXT = new Set([
  "png", "jpg", "jpeg", "jfif", "gif", "apng", "webp", "svg", "bmp",
  "avif", "ico", "heic", "heif", "tif", "tiff",
]);
const VIDEO_EXT = new Set([
  "mp4", "m4v", "mov", "webm", "ogv", "mkv", "avi", "m2ts", "mts",
  "3gp", "3g2", "mpeg", "mpg", "qt", "flv", "wmv",
]);
const AUDIO_EXT = new Set([
  "mp3", "wav", "ogg", "oga", "m4a", "aac", "flac", "opus", "weba",
  "wma", "aiff", "aif", "aifc", "caf", "mid", "midi",
]);

// Formats macOS WKWebView (AVFoundation) reliably decodes in a media element.
// Anything outside this allowlist (webm, mkv, avi, ogg, opus, flac, ProRes, …)
// is shown as an "open externally" card rather than a silently blank player.
const VIDEO_INLINE = new Set(["mp4", "m4v", "mov", "qt"]);
const AUDIO_INLINE = new Set(["mp3", "m4a", "aac", "wav", "aif", "aiff"]);

const extOf = (path: string) => (path.split(".").pop() ?? "").toLowerCase();

// Classify an asset by its extension. Drives both how it embeds in markdown
// (media -> `![]()`, anything else -> `[name]()` link) and how it renders.
export function assetKind(path: string): AssetKind {
  const ext = extOf(path);
  if (IMAGE_EXT.has(ext)) return "image";
  if (VIDEO_EXT.has(ext)) return "video";
  if (AUDIO_EXT.has(ext)) return "audio";
  return "file";
}

// Whether a video/audio asset can play inline in this webview, or must defer to
// the OS. Images never need this — `<img>` handles every format we accept.
export function inlinePlayable(path: string): boolean {
  const ext = extOf(path);
  return VIDEO_INLINE.has(ext) || AUDIO_INLINE.has(ext);
}

// True when a note is nothing but asset refs (whitespace aside) — one or more
// images, videos, audio or files and no prose. Such notes shed the colored
// sticky card and render as their own asset component instead.
export function isAssetOnly(text: string): boolean {
  const t = text.trim();
  if (!t) return false;
  let any = false;
  const rest = t.replace(ASSET_RE, () => {
    any = true;
    return "";
  });
  return any && rest.trim() === "";
}

// Resolve a markdown asset target to something the webview can load: external
// urls/data uris pass through; relative `assets/…` paths get the asset protocol.
export function resolveAssetSrc(raw: string): string {
  if (/^[a-z][a-z0-9+.-]*:/i.test(raw)) return raw;
  return convertFileSrc(assetPath(raw));
}

// Absolute on-disk path for a stored asset, used to hand a file to the OS via
// the opener. External urls pass straight through.
export function assetPath(raw: string): string {
  if (/^[a-z][a-z0-9+.-]*:/i.test(raw)) return raw;
  return `${board.folder.replace(/\/$/, "")}/${raw.replace(/^\.?\//, "")}`;
}
