import { registerIcons } from "@facile/muse";

const line = (body: string) => ({
  body: `<g fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">${body}</g>`,
  width: 24,
  height: 24,
});

/** The canvas glyphs muse does not carry, drawn in the Solar linear weight. */
export const toolIcons = {
  select: "toile:select",
  hand: "toile:hand",
  sticky: "toile:sticky",
  text: "toile:text",
  rect: "toile:rect",
  ellipse: "toile:ellipse",
  diamond: "toile:diamond",
  line: "toile:line",
  arrow: "toile:arrow",
  pen: "toile:pen",
  eraser: "toile:eraser",
  frame: "toile:frame",
  comment: "toile:comment",
  image: "toile:image",
  lock: "toile:lock",
  unlock: "toile:unlock",
  front: "toile:front",
  back: "toile:back",
  fit: "toile:fit",
  share: "toile:share",
  keyboard: "toile:keyboard",
  undo: "toile:undo",
  redo: "toile:redo",
  ai: "toile:ai",
} as const;

registerIcons({
  [toolIcons.ai]: line(
    '<path d="M10 4.5l1.6 4.4 4.4 1.6-4.4 1.6L10 16.5l-1.6-4.4L4 10.5l4.4-1.6z"/><path d="M17.5 14l.8 2.2 2.2.8-2.2.8-.8 2.2-.8-2.2-2.2-.8 2.2-.8z"/><path d="M17 3.5v3M15.5 5h3"/>',
  ),
  [toolIcons.select]: line('<path d="M5 3.5l13 7-5.5 1.6L10 18z"/>'),
  [toolIcons.hand]: line(
    '<path d="M8 12V6a1.5 1.5 0 0 1 3 0v5m0-6.5a1.5 1.5 0 0 1 3 0V11m0-4a1.5 1.5 0 0 1 3 0v6c0 4-2.5 7-6.5 7-3 0-4.5-1.5-6-4l-1.5-3a1.5 1.5 0 0 1 2.6-1.5L8 13"/>',
  ),
  [toolIcons.sticky]: line(
    '<path d="M4 6a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v8l-6 6H6a2 2 0 0 1-2-2z"/><path d="M14 20v-4a2 2 0 0 1 2-2h4"/>',
  ),
  [toolIcons.text]: line('<path d="M5 6V4.5h14V6M12 4.5v15M9 19.5h6"/>'),
  [toolIcons.rect]: line(
    '<rect x="3.5" y="5.5" width="17" height="13" rx="2"/>',
  ),
  [toolIcons.ellipse]: line('<circle cx="12" cy="12" r="8.5"/>'),
  [toolIcons.diamond]: line('<path d="M12 3l9 9-9 9-9-9z"/>'),
  [toolIcons.line]: line('<path d="M5 19L19 5"/>'),
  [toolIcons.arrow]: line('<path d="M5 19L19 5M10 5h9v9"/>'),
  [toolIcons.pen]: line(
    '<path d="M4 20l1-4L16 5a2.1 2.1 0 0 1 3 3L8 19z"/><path d="M14 7l3 3"/>',
  ),
  [toolIcons.eraser]: line(
    '<path d="M8.5 20H20M4.6 15.4l9.8-9.8a2 2 0 0 1 2.8 0l2.2 2.2a2 2 0 0 1 0 2.8L10 20H8.5z"/><path d="M9 11l5 5"/>',
  ),
  [toolIcons.frame]: line('<path d="M7 3v18M17 3v18M3 7h18M3 17h18"/>'),
  [toolIcons.comment]: line(
    '<path d="M12 20c4.7 0 8.5-3.4 8.5-7.5S16.7 5 12 5s-8.5 3.4-8.5 7.5c0 1.8.7 3.4 1.9 4.7L5 21l3.6-1.6c1 .4 2.2.6 3.4.6z"/>',
  ),
  [toolIcons.image]: line(
    '<rect x="3.5" y="4.5" width="17" height="15" rx="2"/><circle cx="9" cy="10" r="1.6"/><path d="M20.5 16l-4.5-4.5L6 19.5"/>',
  ),
  [toolIcons.lock]: line(
    '<rect x="5" y="10.5" width="14" height="10" rx="2"/><path d="M8 10.5V8a4 4 0 0 1 8 0v2.5"/>',
  ),
  [toolIcons.unlock]: line(
    '<rect x="5" y="10.5" width="14" height="10" rx="2"/><path d="M8 10.5V8a4 4 0 0 1 7.7-1.5"/>',
  ),
  [toolIcons.front]: line(
    '<rect x="9" y="9" width="11" height="11" rx="2"/><path d="M15 9V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v7a2 2 0 0 0 2 2h3"/>',
  ),
  [toolIcons.back]: line(
    '<rect x="4" y="4" width="11" height="11" rx="2"/><path d="M9 15v3a2 2 0 0 0 2 2h7a2 2 0 0 0 2-2v-7a2 2 0 0 0-2-2h-3"/>',
  ),
  [toolIcons.fit]: line(
    '<path d="M4 9V5a1 1 0 0 1 1-1h4M15 4h4a1 1 0 0 1 1 1v4M20 15v4a1 1 0 0 1-1 1h-4M9 20H5a1 1 0 0 1-1-1v-4"/>',
  ),
  [toolIcons.share]: line(
    '<circle cx="18" cy="5.5" r="2.5"/><circle cx="6" cy="12" r="2.5"/><circle cx="18" cy="18.5" r="2.5"/><path d="M8.2 10.8l7.6-4M8.2 13.2l7.6 4"/>',
  ),
  [toolIcons.keyboard]: line(
    '<rect x="2.5" y="6" width="19" height="12" rx="2"/><path d="M6 10h.01M10 10h.01M14 10h.01M18 10h.01M8 14h8"/>',
  ),
  [toolIcons.undo]: line(
    '<path d="M9 14L4 9l5-5"/><path d="M4 9h10.5a5.5 5.5 0 0 1 0 11H11"/>',
  ),
  [toolIcons.redo]: line(
    '<path d="M15 14l5-5-5-5"/><path d="M20 9H9.5a5.5 5.5 0 0 0 0 11H13"/>',
  ),
});
