export type Pt = { x: number; y: number };

const r = (n: number) => Math.round(n * 100) / 100;

export function simplify(points: Pt[], min = 1.2): Pt[] {
  if (points.length <= 2) return points;
  const out: Pt[] = [points[0]];
  const min2 = min * min;
  for (let i = 1; i < points.length - 1; i++) {
    const p = points[i];
    const last = out[out.length - 1];
    const dx = p.x - last.x;
    const dy = p.y - last.y;
    if (dx * dx + dy * dy >= min2) out.push(p);
  }
  out.push(points[points.length - 1]);
  return out;
}

export function smoothPath(points: Pt[]): string {
  const p = points;
  if (p.length === 0) return "";
  if (p.length === 1) return `M${r(p[0].x)},${r(p[0].y)}l0.01,0`;
  if (p.length === 2)
    return `M${r(p[0].x)},${r(p[0].y)}L${r(p[1].x)},${r(p[1].y)}`;
  let d = `M${r(p[0].x)},${r(p[0].y)}`;
  for (let i = 1; i < p.length - 1; i++) {
    const mx = (p[i].x + p[i + 1].x) / 2;
    const my = (p[i].y + p[i + 1].y) / 2;
    d += `Q${r(p[i].x)},${r(p[i].y)} ${r(mx)},${r(my)}`;
  }
  const e = p[p.length - 1];
  return d + `L${r(e.x)},${r(e.y)}`;
}
