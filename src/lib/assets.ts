import { convertFileSrc } from "@tauri-apps/api/core";
import { board } from "./board.svelte";
import { isBareUrl } from "./links";

export type AssetKind = "image" | "video" | "audio" | "file";

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

const VIDEO_INLINE = new Set(["mp4", "m4v", "mov", "qt"]);
const AUDIO_INLINE = new Set(["mp3", "m4a", "aac", "wav", "aif", "aiff"]);

const extOf = (path: string) => (path.split(".").pop() ?? "").toLowerCase();

export function assetKind(path: string): AssetKind {
  const ext = extOf(path);
  if (IMAGE_EXT.has(ext)) return "image";
  if (VIDEO_EXT.has(ext)) return "video";
  if (AUDIO_EXT.has(ext)) return "audio";
  return "file";
}

export function inlinePlayable(path: string): boolean {
  const ext = extOf(path);
  return VIDEO_INLINE.has(ext) || AUDIO_INLINE.has(ext);
}

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

// A note that's "just a card" — only embedded assets, or a single bare URL —
// drops the postit chrome and sizes to its content (image, player, link card).
export function isCardOnly(text: string): boolean {
  return isAssetOnly(text) || isBareUrl(text);
}

export function resolveAssetSrc(raw: string): string {
  if (/^[a-z][a-z0-9+.-]*:/i.test(raw)) return raw;
  return convertFileSrc(assetPath(raw));
}

export function assetPath(raw: string): string {
  if (/^[a-z][a-z0-9+.-]*:/i.test(raw)) return raw;
  return `${board.folder.replace(/\/$/, "")}/${raw.replace(/^\.?\//, "")}`;
}
