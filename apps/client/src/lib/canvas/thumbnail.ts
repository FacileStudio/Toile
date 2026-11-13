import type { Box } from "./camera";
import type { Editor } from "./editor.svelte";
import { union } from "./geometry";
import { drawShape, type DrawEnv } from "./render/shapes";

/** The thumbnail size in CSS pixels, the 16:10 of the boards grid. */
export const THUMB = { width: 480, height: 300 };

const PAD = 0.08;

/** The world box a thumbnail frames: every shape, padded, or null on an empty board. */
export function thumbnailBox(boxes: Box[]): Box | null {
  const all = union(boxes);
  if (!all) return null;
  const scale =
    Math.min(THUMB.width / all.w, THUMB.height / all.h) * (1 - PAD * 2);
  const w = THUMB.width / scale;
  const h = THUMB.height / scale;
  return { x: all.x - (w - all.w) / 2, y: all.y - (h - all.h) / 2, w, h };
}

/**
 * Paints every shape of the board into a PNG sized `THUMB` at twice the density; an empty board is a blank page.
 * Remote images load without CORS and would taint the canvas, so cards draw without their banner here.
 */
export function renderThumbnail(editor: Editor): Promise<Blob | null> {
  const boxes: Box[] = [];
  for (const id of editor.scene.shapes.keys()) {
    const box = editor.scene.index.box(id);
    if (box) boxes.push(box);
  }
  const frame = thumbnailBox(boxes);
  const canvas = document.createElement("canvas");
  canvas.width = THUMB.width * 2;
  canvas.height = THUMB.height * 2;
  const ctx = canvas.getContext("2d");
  if (!ctx) return Promise.resolve(null);
  ctx.fillStyle = editor.palette.page;
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  if (!frame) return encode(canvas);
  const scale = canvas.width / frame.w;
  ctx.setTransform(scale, 0, 0, scale, -frame.x * scale, -frame.y * scale);
  const env: DrawEnv = {
    palette: editor.palette,
    lookup: editor.scene.get,
    zoom: scale / 2,
    editing: null,
    image: (asset) =>
      /^https?:/.test(asset) ? null : editor.host.imageSource(asset),
  };
  for (const shape of editor.scene.order) drawShape(ctx, shape, env);
  return encode(canvas);
}

function encode(canvas: HTMLCanvasElement): Promise<Blob | null> {
  return new Promise((resolve) => {
    try {
      canvas.toBlob(resolve, "image/png");
    } catch {
      resolve(null);
    }
  });
}

/** How long the board must stay still before its thumbnail is stored, in milliseconds. */
export const SHOT_DELAY = 3000;

/**
 * Stores the board's thumbnail through `store` once edits pause for `SHOT_DELAY`. `schedule` restarts the wait,
 * `flush` stores a pending one at once; an unchanged scene is never stored twice, and a failed store retries next time.
 */
export function thumbnailer(
  editor: Editor,
  store: (png: Blob) => Promise<unknown>,
) {
  let stored = -1;
  let timer = 0;
  const shoot = () => {
    window.clearTimeout(timer);
    timer = 0;
    if (editor.scene.version === stored) return;
    stored = editor.scene.version;
    void renderThumbnail(editor)
      .then((png) => png && store(png))
      .catch(() => (stored = -1));
  };
  return {
    schedule() {
      window.clearTimeout(timer);
      timer = window.setTimeout(shoot, SHOT_DELAY);
    },
    flush() {
      if (timer) shoot();
    },
  };
}
