import { chromium, type Page } from 'playwright-core';
import { startFake } from './fake';
import { seedBoard } from './seed';

type Phase = { name: string; frames: number[]; paints: number[] };

const quantile = (values: number[], q: number) => {
	const sorted = [...values].sort((a, b) => a - b);
	return sorted[Math.min(sorted.length - 1, Math.floor(q * sorted.length))] ?? 0;
};

async function phase(
	page: Page,
	name: string,
	steps: number,
	wheel: { dx: number; dy: number; ctrl: boolean; flip?: number }
): Promise<Phase> {
	return page.evaluate(
		async ({ name, steps, wheel }) => {
			const target = document.querySelector('[role="application"]') as HTMLElement;
			const paints: number[] = [];
			const observer = new PerformanceObserver((list) => list.getEntries().forEach((e) => paints.push(e.duration)));
			observer.observe({ entryTypes: ['measure'] });
			const frames: number[] = [];
			const next = () => new Promise<number>((resolve) => requestAnimationFrame(resolve));
			let last = await next();
			for (let i = 0; i < steps; i += 1) {
				const sign = wheel.flip && Math.floor(i / wheel.flip) % 2 ? -1 : 1;
				const init = {
					deltaX: wheel.dx * sign,
					deltaY: wheel.dy * sign,
					ctrlKey: wheel.ctrl,
					clientX: 700,
					clientY: 450,
					bubbles: true,
					cancelable: true
				};
				target.dispatchEvent(new WheelEvent('wheel', init));
				const now = await next();
				frames.push(now - last);
				last = now;
			}
			await next();
			observer.disconnect();
			return { name, frames, paints };
		},
		{ name, steps, wheel }
	);
}

const report = (p: Phase) =>
	`${p.name.padEnd(28)} frame p50 ${quantile(p.frames, 0.5).toFixed(1)} ms, p95 ${quantile(p.frames, 0.95).toFixed(1)} ms | paint p50 ${quantile(p.paints, 0.5).toFixed(2)} ms, p95 ${quantile(p.paints, 0.95).toFixed(2)} ms, max ${Math.max(...p.paints).toFixed(2)} ms (${p.paints.length} paints)`;

/** Seeds a board with `count` shapes in the relay, opens it headless and measures pan and zoom frame and paint times. */
async function main(count: number): Promise<void> {
	const stack = startFake();
	seedBoard(stack.relay.room(stack.board).doc, count);
	const browser = await chromium.launch({ headless: true });
	try {
		const context = await browser.newContext({ locale: 'fr-FR', viewport: { width: 1440, height: 900 } });
		await context.addCookies([{ name: 'fake', value: 'alice', url: stack.base }]);
		const page = await context.newPage();
		await page.goto(`${stack.base}/b/${stack.board}`);
		await page.waitForSelector('[data-testid=board][data-editable=true]', { timeout: 30_000 });
		await page.waitForTimeout(1000);
		await page.keyboard.press('Shift+1');
		if (process.env.NO_MINIMAP) await page.keyboard.press('m');
		const results = [
			await phase(page, `pan, all ${count} visible`, 240, { dx: 6, dy: 4, ctrl: false }),
			await phase(page, 'zoom in and out', 240, { dx: 0, dy: -8, ctrl: true, flip: 60 })
		];
		await page.keyboard.press('Shift+0');
		results.push(await phase(page, 'pan at 100 %', 240, { dx: 12, dy: 8, ctrl: false }));
		console.log(`perf: ${count} shapes, ${await browser.version()}, headless`);
		for (const result of results) console.log(report(result));
	} finally {
		await browser.close();
		stack.stop();
	}
}

await main(Number(process.argv[2] ?? 5000));
process.exit(0);
