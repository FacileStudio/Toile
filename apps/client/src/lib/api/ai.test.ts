import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("$app/environment", () => ({ browser: true }));
vi.mock("$app/navigation", () => ({ goto: async () => undefined }));

const { aiPlanLimit, clusterShapes, generateShapes, summarizeBoard } =
  await import("./ai");

afterEach(() => vi.unstubAllGlobals());

const plan = {
  error: {
    code: "payment_required",
    message: "Offre payante",
    upgrade_url: "https://sso.facile.studio/spaces",
    key: "ai.access",
  },
};

describe("ai api", () => {
  it("posts each action with the CSRF header and reads its answer", async () => {
    const fetch = vi.fn(async (url: string) =>
      Response.json(
        url.endsWith("summarize")
          ? { markdown: "# Hi" }
          : url.endsWith("cluster")
            ? { frames: [] }
            : { created: [] },
      ),
    );
    vi.stubGlobal("fetch", fetch);
    expect(await summarizeBoard("b/1")).toBe("# Hi");
    await clusterShapes("b1", ["s1", "s2"]);
    await generateShapes("b1", "Plan", "mindmap");
    const calls = fetch.mock.calls as unknown as [string, RequestInit][];
    expect(calls.map(([url]) => url)).toEqual([
      "/api/boards/b%2F1/ai/summarize",
      "/api/boards/b1/ai/cluster",
      "/api/boards/b1/ai/generate",
    ]);
    expect(new Headers(calls[1][1].headers).get("X-Facile-CSRF")).toBe("1");
    expect(JSON.parse(String(calls[2][1].body))).toEqual({
      prompt: "Plan",
      kind: "mindmap",
    });
  });

  it("reads a 402 as the plan limit and nothing else", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => Response.json(plan, { status: 402 })),
    );
    const limit = await summarizeBoard("b1").catch(aiPlanLimit);
    expect(limit).toMatchObject({
      key: "ai.access",
      upgrade_url: "https://sso.facile.studio/spaces",
    });
    vi.stubGlobal(
      "fetch",
      vi.fn(async () =>
        Response.json(
          { error: { code: "unavailable", message: "x" } },
          { status: 503 },
        ),
      ),
    );
    expect(await summarizeBoard("b1").catch(aiPlanLimit)).toBeNull();
  });
});
