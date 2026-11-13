import { describe, expect, it } from "vitest";
import { THUMB, thumbnailBox } from "./thumbnail";

describe("thumbnailBox", () => {
  it("is null on an empty board", () => {
    expect(thumbnailBox([])).toBeNull();
  });

  it("frames every shape at the grid ratio with the content centred", () => {
    const box = thumbnailBox([
      { x: 0, y: 0, w: 100, h: 100 },
      { x: 300, y: 0, w: 100, h: 100 },
    ])!;
    expect(box.w / box.h).toBeCloseTo(THUMB.width / THUMB.height);
    expect(box.x + box.w / 2).toBeCloseTo(200);
    expect(box.y + box.h / 2).toBeCloseTo(50);
    expect(box.x).toBeLessThan(0);
    expect(box.x + box.w).toBeGreaterThan(400);
  });
});
