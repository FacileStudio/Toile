import { invoke } from "@tauri-apps/api/core";

export type Note = {
  id: string;
  x: number;
  y: number;
  w: number;
  h: number;
  color: string;
  text: string;
  font?: FontKey;
  z: number;
};

export type FontKey = "sans" | "serif" | "mono";

export type Stroke = { id: string; color: string; width: number; d: string };

type StrokeOp = { type: "add" | "erase"; strokes: Stroke[] };

export type Camera = { x: number; y: number; scale: number };

export const TEXT_COLOR = "transparent";

export const COLORS = [
  "#ffe8a3",
  "#ffc9c9",
  "#c9e8ca",
  "#bfe0f2",
  "#e2cdf2",
  "#ffd6b0",
];

export const NOTE_SIZE = 224;
export const MIN_SCALE = 0.2;
export const MAX_SCALE = 3;

const CAMERA_KEY = "toile.camera.v1";

function uid(): string {
  return Math.random().toString(36).slice(2, 10) + Date.now().toString(36);
}

class Board {
  notes = $state<Note[]>([]);
  strokes = $state<Stroke[]>([]);
  camera = $state<Camera>({ x: 0, y: 0, scale: 1 });
  folder = $state("");
  unfurl = $state(true);
  #zTop = 1;
  #undo = $state<StrokeOp[]>([]);
  #redo = $state<StrokeOp[]>([]);
  #strokeTimer: ReturnType<typeof setTimeout> | undefined;

  async init(): Promise<void> {
    if (typeof localStorage !== "undefined") {
      try {
        const raw = localStorage.getItem(CAMERA_KEY);
        if (raw) this.camera = JSON.parse(raw);
      } catch {
      }
    }
    try {
      const data = await invoke<{ folder: string; notes: Note[]; unfurl: boolean }>(
        "init_board",
      );
      this.folder = data.folder;
      this.unfurl = data.unfurl ?? true;
      this.notes = data.notes ?? [];
      this.#zTop = this.notes.reduce((m, n) => Math.max(m, n.z), 1);
    } catch (e) {
      console.error("init_board failed", e);
    }
    try {
      const parsed = JSON.parse(await invoke<string>("load_strokes"));
      if (Array.isArray(parsed?.strokes)) this.strokes = parsed.strokes;
    } catch {
    }
  }

  add(color: string, worldX: number, worldY: number, text = "", font?: FontKey): Note {
    const note: Note = {
      id: uid(),
      x: worldX - NOTE_SIZE / 2,
      y: worldY - NOTE_SIZE / 2,
      w: NOTE_SIZE,
      h: NOTE_SIZE,
      color,
      text,
      font,
      z: ++this.#zTop,
    };
    this.notes.push(note);
    this.writeNote(note.id);
    return note;
  }

  get canUndo(): boolean {
    return this.#undo.length > 0;
  }
  get canRedo(): boolean {
    return this.#redo.length > 0;
  }

  addStroke(d: string, color: string, width: number): void {
    if (!d) return;
    const s: Stroke = { id: uid(), color, width, d };
    this.strokes.push(s);
    this.#undo.push({ type: "add", strokes: [s] });
    this.#redo = [];
    this.#saveStrokes();
  }

  removeStrokes(ids: Set<string>): Stroke[] {
    if (!ids.size) return [];
    const removed = this.strokes.filter((s) => ids.has(s.id));
    if (removed.length) {
      this.strokes = this.strokes.filter((s) => !ids.has(s.id));
      this.#saveStrokes();
    }
    return removed;
  }

  commitErase(removed: Stroke[]): void {
    if (!removed.length) return;
    this.#undo.push({ type: "erase", strokes: removed });
    this.#redo = [];
  }

  #apply(op: StrokeOp, forward: boolean): void {
    const adding = (op.type === "add") === forward;
    if (adding) {
      this.strokes.push(...op.strokes);
    } else {
      const ids = new Set(op.strokes.map((s) => s.id));
      this.strokes = this.strokes.filter((s) => !ids.has(s.id));
    }
  }

  undoStroke(): void {
    const op = this.#undo.pop();
    if (!op) return;
    this.#apply(op, false);
    this.#redo.push(op);
    this.#saveStrokes();
  }

  redoStroke(): void {
    const op = this.#redo.pop();
    if (!op) return;
    this.#apply(op, true);
    this.#undo.push(op);
    this.#saveStrokes();
  }

  #saveStrokes(): void {
    clearTimeout(this.#strokeTimer);
    const snapshot = JSON.stringify({ strokes: this.strokes });
    this.#strokeTimer = setTimeout(() => {
      invoke("save_strokes", { data: snapshot }).catch(() => {});
    }, 400);
  }

  async saveAsset(bytes: Uint8Array, ext: string): Promise<string> {
    return invoke<string>("save_asset", { data: bytes, ext });
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
