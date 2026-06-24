import { invoke } from "@tauri-apps/api/core";

export type Note = {
  id: string;
  x: number;
  y: number;
  w: number;
  h: number;
  color: string;
  text: string;
  z: number;
};

export type Camera = { x: number; y: number; scale: number };

export const COLORS = [
  "#ffe8a3", // butter
  "#ffc9c9", // blush
  "#c9e8ca", // mint
  "#bfe0f2", // sky
  "#e2cdf2", // lavender
  "#ffd6b0", // peach
];

export const NOTE_SIZE = 224;
export const MIN_SCALE = 0.2;
export const MAX_SCALE = 3;

const CAMERA_KEY = "tableau.camera.v1";

function uid(): string {
  return Math.random().toString(36).slice(2, 10) + Date.now().toString(36);
}

class Board {
  notes = $state<Note[]>([]);
  camera = $state<Camera>({ x: 0, y: 0, scale: 1 });
  folder = $state("");
  #zTop = 1;

  async init(): Promise<void> {
    if (typeof localStorage !== "undefined") {
      try {
        const raw = localStorage.getItem(CAMERA_KEY);
        if (raw) this.camera = JSON.parse(raw);
      } catch {
        /* ignore */
      }
    }
    try {
      const data = await invoke<{ folder: string; notes: Note[] }>("init_board");
      this.folder = data.folder;
      this.notes = data.notes ?? [];
      this.#zTop = this.notes.reduce((m, n) => Math.max(m, n.z), 1);
    } catch (e) {
      console.error("init_board failed", e);
    }
  }

  add(color: string, worldX: number, worldY: number, text = ""): Note {
    const note: Note = {
      id: uid(),
      x: worldX - NOTE_SIZE / 2,
      y: worldY - NOTE_SIZE / 2,
      w: NOTE_SIZE,
      h: NOTE_SIZE,
      color,
      text,
      z: ++this.#zTop,
    };
    this.notes.push(note);
    this.writeNote(note.id);
    return note;
  }

  async saveAsset(bytes: Uint8Array, ext: string): Promise<string> {
    return invoke<string>("save_asset", { data: Array.from(bytes), ext });
  }

  remove(id: string): void {
    this.notes = this.notes.filter((n) => n.id !== id);
    invoke("delete_note", { id }).catch(() => {});
  }

  bringToFront(note: Note): void {
    note.z = ++this.#zTop;
  }

  noteZTop(z: number): void {
    this.#zTop = Math.max(this.#zTop, z);
  }

  writeNote(id: string): void {
    const note = this.notes.find((n) => n.id === id);
    if (!note) return;
    invoke("write_note", { note: { ...note } }).catch(() => {});
  }

  saveCamera(): void {
    if (typeof localStorage === "undefined") return;
    localStorage.setItem(CAMERA_KEY, JSON.stringify(this.camera));
  }
}

export const board = new Board();
