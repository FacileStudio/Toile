import type { FontKey } from "./board.svelte";

export const FONT_STACKS: Record<FontKey, string> = {
  sans: '"Goga", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
  serif: '"Iowan Old Style", "Palatino Linotype", Palatino, Georgia, serif',
  mono: 'ui-monospace, "SF Mono", Menlo, "Roboto Mono", monospace',
};
