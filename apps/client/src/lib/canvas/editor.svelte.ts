import type * as Y from 'yjs';
import { BoardDoc } from '$lib/board/doc';
import type { Presence } from '$lib/board/presence';
import type { Style } from '$lib/board/schema';
import { createUndo } from '$lib/board/undo';
import { toWorld, viewport, type Camera, type Point } from './camera';
import { drawOverlay, type OverlayState } from './render/overlay';
import { readPalette, type Palette } from './render/palette';
import { drawScene } from './render/renderer';
import { Scene } from './scene';
import type { ToolName } from './tools/tool';

/** The style new shapes take: `fill` for stickies, `shapeFill` for boxes, `stroke` for lines, ink and text. */
export type ToolStyle = Style & { shapeFill?: string };

/** What the editor asks of its host page. */
export type EditorHost = {
	uploadImage: (file: File) => Promise<{ asset: string; w: number; h: number } | null>;
	imageSource: (asset: string) => CanvasImageSource | null;
	oncomment: (at: Point, shape: string | null) => void;
};

/**
 * One open board on screen: the document, its read model, the camera, the tool, the selection and the overlay.
 * Drawing is batched: any change calls `invalidate`, and one animation frame flushes the scene and repaints.
 */
export class Editor {
	readonly doc: BoardDoc;
	readonly scene: Scene;
	readonly undo: Y.UndoManager;
	tool = $state<ToolName>('select');
	selection = $state.raw<string[]>([]);
	editing = $state<string | null>(null);
	camera = $state.raw<Camera>({ x: 0, y: 0, z: 1 });
	version = $state(0);
	editable = $state(false);
	commentable = $state(false);
	style = $state<ToolStyle>({});
	size = $state.raw({ width: 0, height: 0 });
	overlay: OverlayState = {
		selection: [],
		hover: null,
		marquee: null,
		guides: { x: [], y: [] },
		draft: null,
		target: null,
		peers: []
	};
	palette: Palette = readPalette();
	presence: Presence | null = null;
	onusercamera: (() => void) | null = null;
	private canvas: HTMLCanvasElement | null = null;
	private overlayCanvas: HTMLCanvasElement | null = null;
	private frame = 0;
	private sceneDirty = true;
	private overlayDirty = true;

	constructor(
		ydoc: Y.Doc,
		readonly host: EditorHost
	) {
		this.doc = new BoardDoc(ydoc);
		this.scene = new Scene(this.doc, () => this.invalidate());
		this.undo = createUndo(this.doc);
	}

	/** Binds the two canvases and paints. */
	attach(canvas: HTMLCanvasElement, overlay: HTMLCanvasElement): void {
		this.canvas = canvas;
		this.overlayCanvas = overlay;
		this.resize();
	}

	/** Releases the document observers and the pending frame. */
	destroy(): void {
		cancelAnimationFrame(this.frame);
		this.scene.destroy();
		this.undo.destroy();
	}

	/** Re-reads the canvas size and repaints. */
	resize(): void {
		if (!this.canvas) return;
		this.size = { width: this.canvas.clientWidth, height: this.canvas.clientHeight };
		this.invalidate();
	}

	/** Re-reads the theme tokens and repaints. */
	retheme(): void {
		this.palette = readPalette();
		this.invalidate();
	}

	/** Schedules a full repaint on the next animation frame. */
	invalidate(): void {
		this.sceneDirty = true;
		this.schedule();
	}

	/** Schedules an overlay-only repaint. */
	invalidateOverlay(): void {
		this.overlayDirty = true;
		this.schedule();
	}

	/** Moves the view, publishes the viewport and repaints. A move not made by follow mode calls `onusercamera`. */
	setCamera(camera: Camera, following = false): void {
		if (!following) this.onusercamera?.();
		this.camera = camera;
		this.presence?.set('viewport', viewport(camera, this.size.width, this.size.height));
		this.invalidate();
	}

	/** The world point under a screen point. */
	world(screen: Point): Point {
		return toWorld(this.camera, screen);
	}

	/** Replaces the selection and tells peers. */
	select(ids: string[]): void {
		this.selection = ids;
		this.overlay.selection = ids;
		this.presence?.set('selection', ids);
		this.invalidateOverlay();
	}

	/** Applies queued document changes to the read model now, instead of at the next frame. */
	sync(): void {
		if (!this.scene.flush()) return;
		this.version = this.scene.version;
		this.pruneSelection();
		this.sceneDirty = true;
	}

	/**
	 * Paints now: flushes queued document changes, then the dirty layers. Each scene paint emits a `toile.paint`
	 * measure for a PerformanceObserver; the buffer is cleared at once so a long session keeps nothing.
	 */
	paint(): void {
		this.frame = 0;
		this.sync();
		const started = performance.now();
		const input = { camera: this.camera, palette: this.palette, editing: this.editing, image: this.host.imageSource };
		if (this.sceneDirty && this.canvas) drawScene(this.canvas, this.scene, input);
		if ((this.sceneDirty || this.overlayDirty) && this.overlayCanvas) {
			drawOverlay(this.overlayCanvas, this.scene, this.overlay, this.camera, this.palette);
		}
		if (this.sceneDirty) {
			performance.measure('toile.paint', { start: started });
			performance.clearMeasures('toile.paint');
		}
		this.sceneDirty = false;
		this.overlayDirty = false;
	}

	private schedule(): void {
		if (!this.frame) this.frame = requestAnimationFrame(() => this.paint());
	}

	private pruneSelection(): void {
		const kept = this.selection.filter((id) => this.scene.shapes.has(id));
		if (kept.length !== this.selection.length) this.select(kept);
		if (this.editing && !this.scene.shapes.has(this.editing)) this.editing = null;
	}
}
