import type { Act } from '@facile/mise-en-scene';
import { GRIP, copy, clear, type BoardNodes, type BoardState, type HeroLayout, type StickyId } from './hero';

function cue(): { wait: Promise<void>; go: () => void } {
	let go = () => {};
	const wait = new Promise<void>((resolve) => (go = resolve));
	return { wait, go };
}

/** One loop: the three people work at once, and the payoff holds before it all leaves together. */
export async function perform(act: Act, board: BoardState, nodes: BoardNodes, layout: HeroLayout): Promise<void> {
	const { you, lea, malik } = board.cursors;
	const at = (id: StickyId, [fx, fy]: readonly [number, number]) => act.point(nodes.stickies[id], fx, fy);
	const words = copy();
	const typed = cue();
	const handed = cue();
	Object.assign(you, act.point(nodes.canvas, 0.84, 0.9));
	Object.assign(lea, act.point(nodes.canvas, 0.3, 0.92));
	Object.assign(malik, act.point(nodes.canvas, 0.72, 0.08));
	await act.wait(600);
	for (const cursor of [you, lea, malik]) cursor.visible = true;

	const visitor = async () => {
		await act.move(you, act.point(nodes.tool('sticky'), 0.6, 0.6));
		await act.click(you, () => (board.tool = 'sticky'));
		await act.move(you, at('beta', [0.35, 0.4]));
		await act.click(you, () => Object.assign(board, { placed: true, tool: 'select', picks: { ...board.picks, you: 'beta' } }));
		await act.type(words.beta, (value) => (board.beta = value), 55);
		typed.go();
		await act.move(you, act.point(nodes.tool('arrow'), 0.6, 0.6));
		await act.click(you, () => Object.assign(board, { tool: 'arrow', picks: { ...board.picks, you: null } }));
		await act.move(you, at('export', layout.from), 700);
		you.down = true;
		board.linked = true;
		await act.move(you, at('beta', layout.to));
		you.down = false;
		board.tool = 'select';
	};

	const leaTrack = async () => {
		await act.wait(200);
		const grab = at('onboarding', GRIP.lea);
		await act.move(lea, grab);
		lea.down = true;
		board.picks.lea = 'onboarding';
		await act.wait(160);
		board.moved = true;
		await act.move(lea, { x: grab.x + layout.drag.x, y: grab.y + layout.drag.y });
		lea.down = false;
		await typed.wait;
		await act.move(lea, at('beta', GRIP.note));
		await act.click(lea, () => (board.picks.lea = 'beta'));
		handed.go();
		await act.type(words.note, (value) => (board.note = value), 60);
	};

	const malikTrack = async () => {
		await act.wait(500);
		await act.move(malik, at('export', GRIP.malik));
		await act.click(malik, () => (board.picks.malik = 'export'));
		await act.type(words.line, (value) => (board.line = value), 80);
		await handed.wait;
		await act.move(malik, at('onboarding', [0.5, 0.75]));
		await act.click(malik, () => (board.picks.malik = 'onboarding'));
	};

	await Promise.all([visitor(), leaTrack(), malikTrack()]);
	await act.wait(2200);
	clear(board);
	await act.wait(900);
}
