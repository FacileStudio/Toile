import { marked } from "marked";

marked.use({
  gfm: true,
  breaks: true,
  tokenizer: {
    table() {
      return undefined;
    },
  },
  renderer: {
    checkbox({ checked }) {
      return `<input type="checkbox" class="md-task" data-interactive${
        checked ? " checked" : ""
      } />`;
    },
    // Inline/autolinked URLs stay clickable. `data-interactive` keeps the canvas
    // pointer logic from treating the click as a note drag; Postit opens the href.
    link({ href, text }) {
      const safe = (href ?? "").replace(/"/g, "%22");
      return `<a href="${safe}" class="md-link" data-interactive rel="noreferrer">${text ?? safe}</a>`;
    },
  },
});

export function renderMarkdown(src: string): string {
  return marked.parse(src) as string;
}

const TASK_RE = /^([ \t]*[-*+] +)\[([ xX])\]/gm;

export function toggleTask(text: string, index: number): string {
  let i = -1;
  return text.replace(TASK_RE, (full, prefix: string, mark: string) => {
    i += 1;
    if (i !== index) return full;
    return `${prefix}[${mark.toLowerCase() === "x" ? " " : "x"}]`;
  });
}
