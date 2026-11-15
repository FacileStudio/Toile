import { afterAll, beforeAll, expect, test } from 'bun:test';
import { chromium, type Browser, type Page } from 'playwright-core';
import { startStack, type Person, type Stack } from './stack';

let stack: Stack;
let browser: Browser;

beforeAll(async () => {
	stack = await startStack();
	browser = await chromium.launch({ headless: true });
}, 60_000);

afterAll(async () => {
	await browser?.close();
	await stack?.stop();
});

async function pageAs(who: Person | null, path: string, locale = 'fr-FR'): Promise<Page> {
	const context = await browser.newContext({ locale });
	if (who) await context.addCookies([{ name: 'session', value: stack.cookies[who], url: stack.base }]);
	const page = await context.newPage();
	await page.goto(`${stack.base}${path}`);
	return page;
}

test('a signed-out visitor gets the landing with sign-up and sign-in links', async () => {
	const page = await pageAs(null, '/');
	await page.getByRole('heading', { name: 'Un tableau blanc infini à partager', level: 1 }).waitFor();
	expect(await page.locator('a[href^="/api/auth/oidc"]').count()).toBeGreaterThan(0);
	expect(await page.locator('a[href^="https://sso.facile.studio/signup"]').count()).toBeGreaterThan(0);
	expect(await page.locator('a[href*="/releases/"]').count()).toBe(0);
});

test('a member whose browser is not French gets the boards page in their language', async () => {
	const expected = [
		['en-US', 'New board', 'Boards'],
		['de-DE', 'Neues Board', 'Boards'],
		['es-ES', 'Nueva pizarra', 'Pizarras']
	] as const;
	for (const [locale, button, heading] of expected) {
		const page = await pageAs('alice', '/boards', locale);
		await page.getByRole('button', { name: button }).first().waitFor();
		await page.getByRole('heading', { name: heading, level: 1 }).waitFor();
		const body = await page.locator('body').innerText();
		expect(body).not.toContain('Nouveau tableau');
		expect(body).not.toContain('Tableaux');
		await page.context().close();
	}
});

test('a member creates a board and lands on the list after sign-in', async () => {
	const page = await pageAs('alice', '/');
	await page.waitForURL(`${stack.base}/boards`);
	await page.getByLabel('Nom du tableau').fill('Rétro de lundi');
	await page.getByRole('button', { name: 'Nouveau tableau' }).click();
	await page.getByRole('list', { name: 'Tableaux' }).getByText('Rétro de lundi').waitFor();
	const again = await pageAs('alice', '/boards');
	await again.getByText('Rétro de lundi').waitFor();
});
