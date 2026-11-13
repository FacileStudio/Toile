import { afterAll, beforeAll, expect, test } from 'bun:test';
import { chromium, type Browser, type BrowserContext, type Page } from 'playwright-core';
import { startFake } from './fake';
import { startStack } from './stack';

type Setup = {
	base: string;
	board: string;
	share: (role: 'view' | 'edit') => Promise<string>;
	sign: (c: BrowserContext, who: 'alice' | 'bob') => Promise<void>;
	stop: () => Promise<void> | void;
};

let setup: Setup;
let browser: Browser;

async function realSetup(): Promise<Setup> {
	const stack = await startStack();
	const call = async <T>(path: string, body: unknown): Promise<T> => {
		const answer = await fetch(`${stack.base}/api${path}`, {
			method: 'POST',
			headers: { cookie: `session=${stack.cookies.alice}`, 'content-type': 'application/json', 'x-facile-csrf': '1' },
			body: JSON.stringify(body)
		});
		if (!answer.ok) throw new Error(`${path}: ${answer.status} ${await answer.text()}`);
		return answer.json() as Promise<T>;
	};
	const board = await call<{ id: string }>('/boards', { name: 'Atelier e2e', space_id: stack.space });
	return {
		base: stack.base,
		board: board.id,
		share: async (role) => (await call<{ token: string }>(`/boards/${board.id}/shares`, { role })).token,
		sign: (context, who) => context.addCookies([{ name: 'session', value: stack.cookies[who], url: stack.base }]),
		stop: stack.stop
	};
}

function fakeSetup(): Setup {
	const stack = startFake();
	return {
		base: stack.base,
		board: stack.board,
		share: async (role) => `${role}-token`,
		sign: (context, who) => context.addCookies([{ name: 'fake', value: who, url: stack.base }]),
		stop: stack.stop
	};
}

beforeAll(async () => {
	setup = process.env.CANVAS_E2E_FAKE ? fakeSetup() : await realSetup();
	browser = await chromium.launch({ headless: true });
}, 120_000);

afterAll(async () => {
	await browser?.close();
	await setup?.stop();
});

async function open(who: 'alice' | 'bob' | null, path: string): Promise<Page> {
	const context = await browser.newContext({ locale: 'fr-FR', viewport: { width: 1280, height: 800 } });
	if (who) await setup.sign(context, who);
	const page = await context.newPage();
	await page.goto(`${setup.base}${path}`);
	return page;
}

async function addSticky(page: Page, text: string, at: { x: number; y: number }): Promise<void> {
	await page.getByRole('button', { name: 'Post-it', exact: true }).click();
	await page.mouse.click(at.x, at.y);
	await page.getByRole('textbox', { name: 'Texte' }).fill(text);
	await page.keyboard.press('Escape');
}

async function find(page: Page, text: string): Promise<void> {
	await page.keyboard.press('Control+k');
	await page.getByRole('textbox', { name: 'Rechercher' }).fill(text);
	await page.getByRole('listbox', { name: 'Résultats' }).getByText(text).waitFor({ timeout: 10_000 });
	await page.keyboard.press('Escape');
	await page.getByRole('textbox', { name: 'Rechercher' }).waitFor({ state: 'hidden' });
}

test('two members edit one board and see each other live', async () => {
	const alice = await open('alice', `/b/${setup.board}`);
	const bob = await open('bob', `/b/${setup.board}`);
	for (const page of [alice, bob])
		await page.locator('[data-testid=board][data-editable=true]').waitFor({ timeout: 20_000 });
	await addSticky(alice, 'Idée d’Alice', { x: 400, y: 300 });
	await addSticky(bob, 'Idée de Bob', { x: 800, y: 300 });
	await find(bob, 'Idée d’Alice');
	await find(alice, 'Idée de Bob');
	await alice.mouse.move(500, 500);
	await bob.getByRole('button', { name: 'Suivre alice' }).waitFor({ timeout: 10_000 });
	await bob.keyboard.press('Control+z');
	await bob.keyboard.press('Control+z');
	await alice.keyboard.press('Control+k');
	await alice.getByRole('textbox', { name: 'Rechercher' }).fill('Idée de Bob');
	await alice.getByText('Aucun résultat.').waitFor({ timeout: 10_000 });
	await alice.keyboard.press('Escape');
	await alice.getByRole('textbox', { name: 'Rechercher' }).waitFor({ state: 'hidden' });
	await find(alice, 'Idée d’Alice');
	expect(await alice.getByText('En ligne').count()).toBe(1);
}, 90_000);

test('a guest with a view link sees the board but cannot edit it', async () => {
	const token = await setup.share('view');
	const guest = await open(null, `/s/${token}`);
	await guest.getByLabel('Votre nom').fill('Invitée');
	await guest.getByRole('button', { name: 'Ouvrir le tableau' }).click();
	const board = guest.locator('[data-testid=board]');
	await board.waitFor({ timeout: 20_000 });
	await guest.getByText('En ligne').waitFor({ timeout: 20_000 });
	expect(await board.getAttribute('data-editable')).toBe('false');
	expect(await board.getAttribute('data-role')).toBe('viewer');
	expect(await guest.getByRole('button', { name: 'Post-it', exact: true }).count()).toBe(0);
	await find(guest, 'Idée d’Alice');
	await guest.keyboard.press('s');
	await guest.mouse.click(600, 600);
	await guest.keyboard.type('pirate');
	const alice = await open('alice', `/b/${setup.board}`);
	await alice.locator('[data-testid=board][data-editable=true]').waitFor({ timeout: 20_000 });
	await alice.keyboard.press('Control+k');
	await alice.getByRole('textbox', { name: 'Rechercher' }).fill('pirate');
	await alice.getByText('Aucun résultat.').waitFor();
	expect(await guest.getByRole('textbox', { name: 'Texte' }).count()).toBe(0);
}, 90_000);

test('a pasted youtube link becomes a card that plays in place and the server keeps it', async () => {
	const alice = await open('alice', `/b/${setup.board}`);
	await alice.locator('[data-testid=board][data-editable=true]').waitFor({ timeout: 20_000 });
	await alice.evaluate(() => {
		const data = new DataTransfer();
		data.setData('text/plain', 'https://youtu.be/PAy5EBTnV6Q?list=RDERCkf0j8pPM');
		window.dispatchEvent(new ClipboardEvent('paste', { clipboardData: data }));
	});
	await alice.getByRole('button', { name: 'Lire la vidéo' }).click({ timeout: 10_000 });
	const player = alice.locator('[data-testid=video-player] iframe');
	await player.waitFor({ timeout: 10_000 });
	expect(await player.getAttribute('src')).toBe('https://www.youtube-nocookie.com/embed/PAy5EBTnV6Q?autoplay=1&rel=0');
	expect(await player.getAttribute('referrerpolicy')).toBe('strict-origin-when-cross-origin');
	await alice.waitForTimeout(1_500);
	expect(await alice.getByText('En ligne').count()).toBe(1);
	expect(await alice.locator('[data-testid=board][data-editable=true]').count()).toBe(1);
}, 60_000);

test('a double click on a sticky opens its text for editing', async () => {
	const page = await open('alice', `/b/${setup.board}`);
	await page.locator('[data-testid=board][data-editable=true]').waitFor({ timeout: 20_000 });
	await addSticky(page, 'Brouillon', { x: 300, y: 600 });
	const field = page.getByRole('textbox', { name: 'Texte' });
	await field.waitFor({ state: 'hidden' });
	await page.mouse.dblclick(300, 600);
	await field.waitFor({ timeout: 5_000 });
	expect(await field.inputValue()).toBe('Brouillon');
	const stored = page.waitForResponse((r) => r.url().endsWith('/thumbnail.png') && r.request().method() === 'PUT', {
		timeout: 15_000
	});
	await field.fill('Relu');
	await page.keyboard.press('Escape');
	await find(page, 'Relu');
	expect((await stored).status()).toBe(204);
}, 60_000);
