/** One block of a Markdown answer, rendered as text only: never HTML. */
export type Block =
  | { kind: "heading"; level: 2 | 3 | 4; text: string }
  | { kind: "list"; items: string[] }
  | { kind: "paragraph"; text: string };

const HEADING = /^(#{1,6})\s+(.*)$/;
const ITEM = /^\s*(?:[-*+]|\d+[.)])\s+(.*)$/;

/** Strips the inline marks a summary uses (`**`, `__`, backticks) and keeps the words. */
export function plain(text: string): string {
  return text.replace(/(\*\*|__|`)/g, "").trim();
}

/** Splits `markdown` into headings, lists and paragraphs, the subset an AI summary writes. */
export function blocks(markdown: string): Block[] {
  const out: Block[] = [];
  let paragraph: string[] = [];
  const flush = () => {
    if (paragraph.length)
      out.push({ kind: "paragraph", text: plain(paragraph.join(" ")) });
    paragraph = [];
  };
  for (const line of markdown.split(/\r?\n/)) {
    const heading = HEADING.exec(line);
    const item = ITEM.exec(line);
    if (!line.trim()) flush();
    else if (heading) {
      flush();
      const level = Math.min(Math.max(heading[1].length + 1, 2), 4) as
        2 | 3 | 4;
      out.push({ kind: "heading", level, text: plain(heading[2]) });
    } else if (item) {
      flush();
      const last = out.at(-1);
      if (last?.kind === "list") last.items.push(plain(item[1]));
      else out.push({ kind: "list", items: [plain(item[1])] });
    } else paragraph.push(line.trim());
  }
  flush();
  return out;
}
