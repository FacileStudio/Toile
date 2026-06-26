// Pure URL helpers — no Tauri, no Svelte. Shared by the link cache store, the
// postit segmenter, and the link card so detection logic lives in one place.

const YT_HOSTS = new Set([
  "youtube.com",
  "m.youtube.com",
  "music.youtube.com",
  "youtube-nocookie.com",
  "youtu.be",
]);

// A standalone URL on its own line becomes a rich card; a URL inside prose stays
// an inline link (the Notion/Discord convention). "Bare" = the whole string is a
// single http(s) token with no surrounding text.
export function isBareUrl(text: string): boolean {
  const t = text.trim();
  return /^https?:\/\/\S+$/i.test(t);
}

export function youtubeId(raw: string): string | null {
  let u: URL;
  try {
    u = new URL(raw);
  } catch {
    return null;
  }
  const host = u.hostname.replace(/^www\./, "").toLowerCase();
  if (!YT_HOSTS.has(host)) return null;

  let id = "";
  if (host === "youtu.be") {
    id = u.pathname.split("/").filter(Boolean)[0] ?? "";
  } else if (u.searchParams.get("v")) {
    id = u.searchParams.get("v") ?? "";
  } else {
    const [head, next] = u.pathname.split("/").filter(Boolean);
    if (head && ["shorts", "embed", "v", "live"].includes(head)) id = next ?? "";
  }
  return /^[a-zA-Z0-9_-]{11}$/.test(id) ? id : null;
}

// Only hand these schemes to the OS opener — never `javascript:`, `data:`, etc.
export function safeExternal(url: string): boolean {
  return /^(https?|mailto):/i.test(url.trim());
}

export function hostOf(url: string): string {
  try {
    return new URL(url).hostname.replace(/^www\./, "");
  } catch {
    return url;
  }
}
