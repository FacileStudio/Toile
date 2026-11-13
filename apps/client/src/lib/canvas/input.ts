import { panBy, pinch, zoomAt, type Point } from './camera';
import type { Editor } from './editor.svelte';
import { HandTool } from './tools/hand';
import { makeTool } from './tools/index';
import type { PointerSample, Tool, ToolName } from './tools/tool';

const CLICK_MS = 500;
const CLICK_SLOP = 6;

/**
 * Turns pointer, wheel and Space events on the board element into camera moves and tool gestures. Two touches pinch
 * and pan; the middle button, Space or a finger after a pen has been seen pan; everything else goes to the tool.
 * Browsers leave `detail` at 0 on `pointerdown`, so the click count tools read comes from `countClick`.
 */
export class Input {
	private pointers = new Map<number, Point>();
	private gesture: Tool | null = null;
	private gestureId: number | null = null;
	private pinchFrom: [Point, Point] | null = null;
	private tool: Tool;
	private space = false;
	private penSeen = false;
	private teardown: (() => void)[] = [];
	private hostRect: DOMRect | null = null;
	private wheelDelta = { x: 0, y: 0 };
	private wheelZoom: { factor: number; screen: Point } | null = null;
	private wheelRaf = 0;
	private clicks = { count: 0, at: 0, x: 0, y: 0 };

	constructor(
		private editor: Editor,
		private element: HTMLElement
	) {
		this.tool = makeTool(editor, editor.tool);
	}

	/** The cursor the board should show. */
	get cursor(): string {
		return this.space ? 'grab' : (this.gesture ?? this.tool).cursor;
	}

	/** Switches the active tool, cancelling any gesture in progress. */
	setTool(name: ToolName): void {
		this.cancel();
		this.tool.leave?.();
		this.tool = makeTool(this.editor, name);
		this.editor.overlay.target = null;
		this.editor.overlay.hover = null;
		this.editor.invalidateOverlay();
	}

	/** Cancels the gesture in progress. */
	cancel(): void {
		this.gesture?.cancel();
		this.gesture = null;
		this.gestureId = null;
	}

	/** Starts `tool` as the gesture of pointer event `event`; used by the selection handles. */
	startGesture(tool: Tool, event: PointerEvent): void {
		this.cancel();
		this.gesture = tool;
		this.gestureId = event.pointerId;
		this.pointers.set(event.pointerId, this.screen(event));
		tool.down(this.sample(event));
	}

	/** Adds the listeners; returns their removal. */
	listen(): () => void {
		const on = <K extends keyof WindowEventMap>(
			target: Window | HTMLElement,
			type: K,
			fn: (e: WindowEventMap[K]) => void,
			options?: AddEventListenerOptions
		) => {
			target.addEventListener(type, fn as EventListener, options);
			this.teardown.push(() => target.removeEventListener(type, fn as EventListener, options));
		};
		on(this.element, 'pointerdown', (e) => this.down(e));
		on(window, 'pointermove', (e) => this.move(e));
		on(this.element, 'pointerleave', () => this.tool.leave?.());
		on(window, 'pointerup', (e) => this.up(e));
		on(window, 'pointercancel', (e) => this.up(e, true));
		on(this.element, 'wheel', (e) => this.wheel(e), { passive: false });
		on(window, 'keydown', (e) => this.key(e, true));
		on(window, 'keyup', (e) => this.key(e, false));
		on(window, 'blur', () => (this.space = false));
		on(window, 'resize', () => (this.hostRect = null));
		on(window, 'scroll', () => (this.hostRect = null), { capture: true, passive: true });
		return () => {
			if (this.wheelRaf) cancelAnimationFrame(this.wheelRaf);
			this.teardown.splice(0).forEach((off) => off());
		};
	}

	private rect(): DOMRect {
		return (this.hostRect ??= this.element.getBoundingClientRect());
	}

	private screen(event: PointerEvent | WheelEvent): Point {
		const r = this.rect();
		return { x: event.clientX - r.left, y: event.clientY - r.top };
	}

	private sample(event: PointerEvent): PointerSample {
		const screen = this.screen(event);
		const r = this.rect();
		const coalesced =
			event.getCoalescedEvents?.().map((e) => ({ x: e.clientX - r.left, y: e.clientY - r.top })) ?? [];
		return {
			world: this.editor.world(screen),
			screen,
			pressure: event.pressure,
			pointerType: event.pointerType,
			shift: event.shiftKey,
			alt: event.altKey,
			meta: event.metaKey || event.ctrlKey,
			detail: event.type === 'pointerdown' ? this.clicks.count : event.detail,
			coalesced
		};
	}

	private down(event: PointerEvent): void {
		this.hostRect = this.element.getBoundingClientRect();
		const target = event.target as Element | null;
		const inText = target instanceof HTMLElement && (target.tagName === 'TEXTAREA' || target.tagName === 'INPUT');
		if (this.editor.editing && !inText) {
			this.editor.finishEditing();
			if (document.activeElement instanceof HTMLElement) document.activeElement.blur();
		}
		if (target?.closest?.('[data-board-ui]')) return;
		if (event.pointerType === 'pen') this.penSeen = true;
		this.countClick(event);
		this.pointers.set(event.pointerId, this.screen(event));
		if (this.pointers.size === 2 && event.pointerType === 'touch') return this.startPinch();
		if (this.gesture || this.pointers.size > 1) return;
		const panning =
			event.button === 1 || this.space || (this.penSeen && event.pointerType === 'touch' && this.editor.tool === 'pen');
		if (event.button !== 0 && !panning) return;
		event.preventDefault();
		this.element.setPointerCapture?.(event.pointerId);
		this.gesture = panning ? new HandTool(this.editor) : this.tool;
		this.gestureId = event.pointerId;
		this.gesture.down(this.sample(event));
	}

	private countClick(event: PointerEvent): void {
		const near = Math.hypot(event.clientX - this.clicks.x, event.clientY - this.clicks.y) <= CLICK_SLOP;
		const soon = event.timeStamp - this.clicks.at <= CLICK_MS;
		this.clicks = {
			count: near && soon ? this.clicks.count + 1 : 1,
			at: event.timeStamp,
			x: event.clientX,
			y: event.clientY
		};
	}

	private startPinch(): void {
		this.cancel();
		const [a, b] = [...this.pointers.values()];
		this.pinchFrom = [a, b];
	}

	private move(event: PointerEvent): void {
		if (
			!this.pointers.has(event.pointerId) &&
			event.target !== this.element &&
			!this.element.contains(event.target as Node)
		)
			return;
		const screen = this.screen(event);
		if (this.pointers.has(event.pointerId)) this.pointers.set(event.pointerId, screen);
		if (this.pinchFrom && this.pointers.size === 2) {
			const now = [...this.pointers.values()] as [Point, Point];
			this.editor.setCamera(pinch(this.editor.camera, this.pinchFrom, now));
			this.pinchFrom = now;
			return;
		}
		this.editor.presence?.set('cursor', this.editor.world(screen));
		if (this.gesture && event.pointerId === this.gestureId) this.gesture.move(this.sample(event));
		else if (!this.gesture && !this.pointers.size) this.tool.hover?.(this.sample(event));
	}

	private up(event: PointerEvent, cancelled = false): void {
		this.pointers.delete(event.pointerId);
		if (this.pointers.size < 2) this.pinchFrom = null;
		if (!this.gesture || event.pointerId !== this.gestureId) return;
		const gesture = this.gesture;
		this.gesture = null;
		this.gestureId = null;
		if (cancelled) gesture.cancel();
		else gesture.up(this.sample(event));
		this.editor.undo.stopCapturing();
	}

	private wheel(event: WheelEvent): void {
		event.preventDefault();
		const screen = this.screen(event);
		if (event.ctrlKey || event.metaKey) {
			const factor = wheelFactor(event.deltaY, event.deltaMode, event.ctrlKey);
			if (this.wheelZoom) {
				this.wheelZoom.factor *= factor;
				this.wheelZoom.screen = screen;
			} else {
				this.wheelZoom = { factor, screen };
			}
		} else {
			const scale = event.deltaMode === 1 ? 16 : 1;
			const [dx, dy] = event.shiftKey && !event.deltaX ? [event.deltaY, 0] : [event.deltaX, event.deltaY];
			this.wheelDelta.x -= dx * scale;
			this.wheelDelta.y -= dy * scale;
		}
		if (!this.wheelRaf) this.wheelRaf = requestAnimationFrame(() => this.flushWheel());
	}

	private flushWheel(): void {
		this.wheelRaf = 0;
		let cam = this.editor.camera;
		if (this.wheelDelta.x || this.wheelDelta.y) {
			cam = panBy(cam, this.wheelDelta.x, this.wheelDelta.y);
			this.wheelDelta.x = 0;
			this.wheelDelta.y = 0;
		}
		if (this.wheelZoom) {
			cam = zoomAt(cam, this.wheelZoom.screen, this.wheelZoom.factor);
			this.wheelZoom = null;
		}
		this.editor.setCamera(cam);
	}

	private key(event: KeyboardEvent, down: boolean): void {
		if (event.code !== 'Space' || isTyping(event.target)) return;
		event.preventDefault();
		this.space = down;
	}
}

/** Whether a key event lands in a text field, where board shortcuts must not fire. */
export function isTyping(target: EventTarget | null): boolean {
	if (!(target instanceof HTMLElement)) return false;
	return target.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName);
}

/** The zoom factor of one wheel event: `ctrlKey` wheels are trackpad pinches and step finer. */
export function wheelFactor(deltaY: number, deltaMode: number, pinching: boolean): number {
	const pixels = deltaMode === 1 ? deltaY * 16 : deltaY;
	return Math.exp(-pixels * (pinching ? 0.01 : 0.002));
}
