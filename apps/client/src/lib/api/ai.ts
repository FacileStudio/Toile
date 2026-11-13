import { ApiError } from "@facile/client";
import { planLimitFrom, type PlanLimitError } from "@facile/muse";
import { api } from "$lib/backend";

/** What `POST /api/boards/{id}/ai/generate` draws. */
export type DrawKind = "mindmap" | "flowchart" | "stickies";

/** A frame the cluster made and the stickies moved into it. */
export type Framed = { frame: string; title: string; shapes: string[] };

/** A shape the generator placed. */
export type Placed = {
  id: string;
  type: string;
  x: number;
  y: number;
  w: number;
  h: number;
};

const path = (board: string, action: string) =>
  `/boards/${encodeURIComponent(board)}/ai/${action}`;

/** Bulle's Markdown summary of board `board`. */
export async function summarizeBoard(board: string): Promise<string> {
  return (
    await api<{ markdown: string }>(path(board, "summarize"), {
      method: "POST",
    })
  ).markdown;
}

/** Frames the selected stickies of `board` by theme; resolves the new frames. */
export async function clusterShapes(
  board: string,
  shapes: string[],
): Promise<Framed[]> {
  const answer = await api<{ frames: Framed[] }>(path(board, "cluster"), {
    method: "POST",
    body: JSON.stringify({ shapes }),
  });
  return answer.frames;
}

/** Draws a `kind` from `prompt` right of the board's content, with `accent` (`#rrggbb`) for its highlights; resolves the new shapes. */
export async function generateShapes(
  board: string,
  prompt: string,
  kind: DrawKind,
  accent?: string,
): Promise<Placed[]> {
  const answer = await api<{ created: Placed[] }>(path(board, "generate"), {
    method: "POST",
    body: JSON.stringify({ prompt, kind, accent }),
  });
  return answer.created;
}

const names = new Map<string, Promise<string>>();

/** The caller's Bulle assistant name, asked once per board; "Bulle" when it cannot be read. */
export function assistantName(board: string): Promise<string> {
  let name = names.get(board);
  if (!name) {
    name = api<{ name: string }>(path(board, "assistant"))
      .then((answer) => answer.name || "Bulle")
      .catch(() => "Bulle");
    names.set(board, name);
  }
  return name;
}

/** The plan refusal of a failed AI call, for muse `PlanLimit`, or `null` for any other failure. */
export function aiPlanLimit(error: unknown): PlanLimitError | null {
  if (!(error instanceof ApiError) || error.status !== 402) return null;
  return (
    planLimitFrom(error.details) ?? {
      message: error.message,
      upgrade_url: error.upgradeUrl,
    }
  );
}

/** The toast for a failed AI call that is not a plan refusal. */
export function aiError(error: unknown): string {
  if (error instanceof ApiError && error.status === 429)
    return "Trop de demandes à l’IA pour le moment. Réessayez plus tard.";
  return error instanceof ApiError
    ? error.message
    : "L’IA n’a pas pu répondre.";
}
