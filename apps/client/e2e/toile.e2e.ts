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

async function pageAs(who: Person | null, path: string): Promise<Page> {
	const context = await browser.newContext({ locale: 'fr-FR' });
	if (who) await context.addCookies([{ name: 'session', value: stack.cookies[who], url: stack.base }]);
	const page = await context.newPage();
	await page.goto(`${stack.base}${path}`);
	return page;
}

test('a signed-out visitor gets the landing with a sign-in link', async () => {
	const page = await pageAs(null, '/');
	await page.getByRole('heading', { name: 'Toile', level: 1 }).waitFor();
	expect(await page.locator('a[href^="/api/auth/oidc"]').count()).toBeGreaterThan(0);
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
