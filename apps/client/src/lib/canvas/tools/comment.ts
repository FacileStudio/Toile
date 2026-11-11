import type { Editor } from '../editor.svelte';
import { topHit } from '../hit';
import type { PointerSample, Tool } from './tool';

/** A click drops a comment: anchored to the shape under it when there is one. */
export class CommentTool implements Tool {
	cursor = 'copy';

	constructor(private editor: Editor) {}

	down(): void {}

	move(): void {}

	up(sample: PointerSample): void {
		const shape = topHit(this.editor.scene, sample.world, 4 / this.editor.camera.z);
		this.editor.host.oncomment(sample.world, shape?.id ?? null);
	}

	cancel(): void {}
}
