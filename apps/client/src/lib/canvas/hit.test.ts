import { describe, expect, it } from 'vitest';
import { BoardDoc } from '$lib/board/doc';
import { frameAt, hitShape, marqueeHits, topHit } from './hit';
import { Scene } from './scene';
import { SpatialIndex } from './spatial';

function board() {
	const doc = new BoardDoc();
	const scene = new Scene(doc, () => undefined);
	return { doc, scene };
}

describe('spatial index', () => {
	it('finds overlapping boxes, forgets deleted ones and handles huge ones', () => {
		const index = new SpatialIndex();
		index.set('a', { x: 0, y: 0, w: 10, h: 10 });
		index.set('b', { x: 5000, y: 5000, w: 10, h: 10 });
		index.set('huge', { x: -1e6, y: -1e6, w: 2e6, h: 2e6 });
		expect([...index.query({ x: -1, y: -1, w: 2, h: 2 })].sort()).toEqual(['a', 'huge']);
		index.set('a', { x: 4990, y: 4990, w: 20, h: 20 });
		expect([...index.query({ x: 4995, y: 4995, w: 10, h: 10 })].sort()).toEqual(['a', 'b', 'huge']);
		index.delete('b');
		expect(index.query({ x: 5005, y: 5005, w: 1, h: 1 }).has('b')).toBe(false);
		expect(index.size).toBe(2);
	});
});

describe('scene', () => {
	it('serves fresh snapshots in z order after a change that keeps z', () => {
		const { doc, scene } = board();
		const id = doc.create('sticky', { x: 0, y: 0, w: 10, h: 10, z: 'V' }, 'a');
		scene.flush();
		expect(scene.order[0].text).toBe('a');
		doc.transact(() => doc.text(id)?.insert(1, 'b'));
		scene.flush();
		expect(scene.order[0].text).toBe('ab');
		expect(scene.within({ x: 0, y: 0, w: 1, h: 1 })[0].text).toBe('ab');
	});
});

describe('hit testing', () => {
	it('hits the topmost shape by z, and respects ellipse and diamond outlines', () => {
		const { doc, scene } = board();
		const low = doc.create('rect', { x: 0, y: 0, w: 100, h: 100, z: 'A' });
		const high = doc.create('ellipse', { x: 0, y: 0, w: 100, h: 100, z: 'B' });
		scene.flush();
		expect(topHit(scene, { x: 50, y: 50 }, 1)?.id).toBe(high);
		expect(topHit(scene, { x: 3, y: 3 }, 1)?.id).toBe(low);
		const diamond = scene.get(doc.create('diamond', { x: 200, y: 0, w: 100, h: 100, z: 'C' }));
		scene.flush();
		expect(diamond).toBeUndefined();
		const shape = scene.get([...scene.shapes.keys()].find((id) => scene.get(id)?.type === 'diamond') as string);
		expect(shape && hitShape(shape, { x: 250, y: 50 }, 0, scene.get)).toBe(true);
		expect(shape && hitShape(shape, { x: 205, y: 5 }, 0, scene.get)).toBe(false);
	});

	it('hits a rotated box in its own frame', () => {
		const { doc, scene } = board();
		const id = doc.create('rect', { x: 0, y: 40, w: 100, h: 20, z: 'A', rot: Math.PI / 2 });
		scene.flush();
		expect(topHit(scene, { x: 50, y: 5 }, 0)?.id).toBe(id);
		expect(topHit(scene, { x: 5, y: 50 }, 0)).toBeNull();
	});

	it('hits ink and arrows near their stroke only, arrows following bound shapes', () => {
		const { doc, scene } = board();
		const ink = doc.create('ink', {
			x: 0,
			y: 0,
			w: 100,
			h: 0,
			z: 'A',
			points: [0, 0, 0.5, 100, 0, 0.5],
			style: { width: 4 }
		});
		const box = doc.create('rect', { x: 500, y: 500, w: 100, h: 100, z: 'B' });
		const arrow = doc.create('arrow', {
			x: 0,
			y: 200,
			w: 100,
			h: 0,
			z: 'C',
			points: [0, 0, 100, 0],
			to: { shape: box, anchor: [0, 0.5] }
		});
		scene.flush();
		expect(topHit(scene, { x: 50, y: 3 }, 2)?.id).toBe(ink);
		expect(topHit(scene, { x: 50, y: 30 }, 2)).toBeNull();
		expect(topHit(scene, { x: 250, y: 375 }, 2)?.id).toBe(arrow);
		doc.update(box, { y: 0 });
		scene.flush();
		expect(scene.index.box(arrow)?.y).toBeLessThan(60);
	});

	it('selects frames by label or edge, by marquee only when inside, and finds the frame under a point', () => {
		const { doc, scene } = board();
		const frame = doc.create('frame', { x: 0, y: 0, w: 400, h: 300, z: 'A', name: 'F' });
		const sticky = doc.create('sticky', { x: 100, y: 100, w: 50, h: 50, z: 'B' });
		const locked = doc.create('rect', { x: 200, y: 100, w: 50, h: 50, z: 'C', locked: true });
		scene.flush();
		expect(topHit(scene, { x: 10, y: -10 }, 1)?.id).toBe(frame);
		expect(topHit(scene, { x: 300, y: 250 }, 1)).toBeNull();
		expect(marqueeHits(scene, { x: 90, y: 90, w: 200, h: 100 }).map((s) => s.id)).toEqual([sticky]);
		expect(marqueeHits(scene, { x: -10, y: -40, w: 500, h: 400 }).map((s) => s.id)).toEqual([frame, sticky]);
		expect(frameAt(scene, { x: 120, y: 120 }, new Set())?.id).toBe(frame);
		expect(frameAt(scene, { x: 120, y: 120 }, new Set([frame]))).toBeNull();
		expect(locked).toBeTruthy();
	});
});
