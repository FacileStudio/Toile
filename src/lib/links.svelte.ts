import { invoke } from "@tauri-apps/api/core";

// What the Rust `fetch_link_preview` command returns (camelCase via serde).
// `image`/`favicon` are board-relative asset paths the backend already cached to
// disk, so cards work offline and don't re-fetch the remote on every render.
export type LinkMeta = {
  url: string;
  kind: "youtube" | "card" | "error";
  title?: string | null;
  description?: string | null;
  siteName?: string | null;
  image?: string | null;
  favicon?: string | null;
  accent?: string | null;
  videoId?: string | null;
};

// Metadata is fetched once and cached forever in `.toile-links.json`; the network
// only gets hit again on an explicit refresh. Errors stay in memory only, so a
// flaky connection retries next session instead of poisoning the card permanently.
class LinkStore {
  cache = $state<Record<string, LinkMeta>>({});
  enabled = $state(true);

  #pending = new Map<string, Promise<LinkMeta | null>>();
  #queue: Array<() => void> = [];
  #active = 0;
  #max = 4;
  #saveTimer: ReturnType<typeof setTimeout> | undefined;

  async init(enabled: boolean): Promise<void> {
    this.enabled = enabled;
    try {
      const data = JSON.parse(await invoke<string>("load_links"));
      if (data && typeof data === "object") this.cache = data;
    } catch {
      /* no cache yet */
    }
  }

  get(url: string): LinkMeta | undefined {
    return this.cache[url];
  }

  // Returns cached metadata, or queues a fetch and resolves when it lands. A no-op
  // (resolves null) when previews are disabled and nothing is cached.
  ensure(url: string): Promise<LinkMeta | null> {
    const hit = this.cache[url];
    if (hit) return Promise.resolve(hit);
    if (!this.enabled) return Promise.resolve(null);

    const inflight = this.#pending.get(url);
    if (inflight) return inflight;

    const job = new Promise<LinkMeta | null>((resolve) => {
      this.#queue.push(() => {
        invoke<LinkMeta>("fetch_link_preview", { url })
          .then((meta) => {
            this.cache[url] = meta;
            this.#persist();
            resolve(meta);
          })
          .catch((e) => {
            this.cache[url] = { url, kind: "error", title: String(e) };
            resolve(this.cache[url]);
          })
          .finally(() => {
            this.#active -= 1;
            this.#pending.delete(url);
            this.#drain();
          });
      });
    });
    this.#pending.set(url, job);
    this.#drain();
    return job;
  }

  refresh(url: string): Promise<LinkMeta | null> {
    delete this.cache[url];
    this.#pending.delete(url);
    return this.ensure(url);
  }

  #drain(): void {
    while (this.#active < this.#max && this.#queue.length) {
      this.#active += 1;
      this.#queue.shift()!();
    }
  }

  #persist(): void {
    clearTimeout(this.#saveTimer);
    // Don't persist transient errors — only real previews are worth caching.
    const keep = Object.fromEntries(
      Object.entries(this.cache).filter(([, m]) => m.kind !== "error"),
    );
    const snapshot = JSON.stringify(keep);
    this.#saveTimer = setTimeout(() => {
      invoke("save_links", { data: snapshot }).catch(() => {});
    }, 400);
  }
}

export const links = new LinkStore();
