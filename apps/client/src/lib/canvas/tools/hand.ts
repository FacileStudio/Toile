import { panBy, type Point } from '../camera';
import type { Editor } from '../editor.svelte';
import type { PointerSample, Tool } from './tool';

/** Drags the view. */
export class HandTool implements Tool {
	cursor = 'grab';
	private last: Point | null = null;

	constructor(private editor: Editor) {}

	down(sample: PointerSample): void {
		this.last = sample.screen;
		this.cursor = 'grabbing';
	}

	move(sample: PointerSample): void {
		if (!this.last) return;
		this.editor.setCamera(panBy(this.editor.camera, sample.screen.x - this.last.x, sample.screen.y - this.last.y));
		this.last = sample.screen;
	}

	up(): void {
		this.cancel();
	}

	cancel(): void {
		this.last = null;
		this.cursor = 'grab';
	}
}
