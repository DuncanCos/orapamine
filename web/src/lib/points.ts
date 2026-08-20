// Réplique côté client la disposition des 36 points de tir générée par
// `orapa_core::beam::entry_points` (lettres A-R à gauche puis en bas,
// chiffres 1-18 en haut puis à droite). Doit rester en phase avec
// `crates/orapa-core/src/beam.rs::entry_points`.

export type Side = "left" | "bottom" | "top" | "right";

export interface BoardPoint {
  id: string;
  cellX: number;
  cellY: number;
  side: Side;
}

const LETTERS = "ABCDEFGHIJKLMNOPQR";

export function boardPoints(width: number, height: number): BoardPoint[] {
  const points: BoardPoint[] = [];
  let li = 0;
  for (let y = 0; y < height; y++) {
    points.push({ id: LETTERS[li++], cellX: 0, cellY: y, side: "left" });
  }
  for (let x = 0; x < width; x++) {
    points.push({ id: LETTERS[li++], cellX: x, cellY: height - 1, side: "bottom" });
  }
  for (let x = 0; x < width; x++) {
    points.push({ id: String(x + 1), cellX: x, cellY: 0, side: "top" });
  }
  for (let y = 0; y < height; y++) {
    points.push({ id: String(width + y + 1), cellX: width - 1, cellY: y, side: "right" });
  }
  return points;
}
