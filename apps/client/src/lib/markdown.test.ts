import { describe, expect, it } from "vitest";
import { blocks } from "./markdown";

describe("blocks", () => {
  it("reads headings, lists and paragraphs as text", () => {
    const md =
      "# Bilan\nLe **projet** avance.\nVite.\n\n- Prix\n- `Délais`\n## Questions\n1. <b>Qui ?</b>";
    expect(blocks(md)).toEqual([
      { kind: "heading", level: 2, text: "Bilan" },
      { kind: "paragraph", text: "Le projet avance. Vite." },
      { kind: "list", items: ["Prix", "Délais"] },
      { kind: "heading", level: 3, text: "Questions" },
      { kind: "list", items: ["<b>Qui ?</b>"] },
    ]);
  });
});
