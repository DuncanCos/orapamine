import { boardPoints } from "../lib/points";
import { GEM_COLORS, SPECIAL_COLORS } from "../lib/colors";
import type { CellKind, GemColor, PieceCatalog, PlacedPiece } from "../types/protocol";

const CELL = 34;
const MARGIN = 26;

// Sommets (en coordonnées unité, 0..1) du polygone couvrant la demi-case
// adjacente à chaque coin — voir la convention dans
// crates/orapa-core/src/geometry.rs (Corner::legs / hypotenuse_diagonal).
const TRIANGLE_POINTS: Record<string, [number, number][]> = {
  nw: [
    [0, 0],
    [1, 0],
    [0, 1],
  ],
  ne: [
    [1, 0],
    [0, 0],
    [1, 1],
  ],
  se: [
    [1, 1],
    [0, 1],
    [1, 0],
  ],
  sw: [
    [0, 1],
    [1, 1],
    [0, 0],
  ],
};

function shapeFill(color: GemColor | null, special: "absorb" | "transparent" | null): string {
  if (color) return GEM_COLORS[color];
  if (special === "absorb") return SPECIAL_COLORS.absorb;
  if (special === "transparent") return SPECIAL_COLORS.transparent;
  return "#999";
}

export interface ProbeMarker {
  x: number;
  y: number;
  kind: "empty" | "occupied_no_color" | "color";
  color?: GemColor;
}

export interface PointHighlight {
  id: string;
  role: "entry" | "exit";
}

export interface BoardProps {
  catalog: PieceCatalog;
  placements: PlacedPiece[];
  showPoints?: boolean;
  usedPointIds?: Set<string>;
  onPointClick?: (id: string) => void;
  onCellClick?: (x: number, y: number) => void;
  onPieceClick?: (index: number) => void;
  selectedIndex?: number | null;
  faultyIndices?: Set<number>;
  markers?: ProbeMarker[];
  highlightPoints?: PointHighlight[];
}

export function Board({
  catalog,
  placements,
  showPoints = false,
  usedPointIds,
  onPointClick,
  onCellClick,
  onPieceClick,
  selectedIndex = null,
  faultyIndices,
  markers,
  highlightPoints,
}: BoardProps) {
  const w = catalog.grid_width;
  const h = catalog.grid_height;
  const ox = MARGIN;
  const oy = MARGIN;
  const viewW = ox * 2 + w * CELL;
  const viewH = oy * 2 + h * CELL;
  const points = showPoints ? boardPoints(w, h) : [];
  const highlightById = new Map((highlightPoints ?? []).map((p) => [p.id, p.role]));

  return (
    <svg
      className="board-svg"
      viewBox={`0 0 ${viewW} ${viewH}`}
      role="group"
      aria-label="Plateau de jeu"
    >
      <rect x={ox} y={oy} width={w * CELL} height={h * CELL} className="board-bg" />

      {onCellClick &&
        Array.from({ length: w * h }, (_, i) => {
          const x = i % w;
          const y = Math.floor(i / w);
          return (
            <rect
              key={`hit-${x}-${y}`}
              x={ox + x * CELL}
              y={oy + y * CELL}
              width={CELL}
              height={CELL}
              className="board-cell-hit"
              onClick={() => onCellClick(x, y)}
            />
          );
        })}

      {/* Grille */}
      {Array.from({ length: w + 1 }, (_, i) => (
        <line
          key={`v${i}`}
          x1={ox + i * CELL}
          y1={oy}
          x2={ox + i * CELL}
          y2={oy + h * CELL}
          className="board-grid-line"
        />
      ))}
      {Array.from({ length: h + 1 }, (_, i) => (
        <line
          key={`h${i}`}
          x1={ox}
          y1={oy + i * CELL}
          x2={ox + w * CELL}
          y2={oy + i * CELL}
          className="board-grid-line"
        />
      ))}

      {/* Pièces posées */}
      {placements.map((p, idx) => {
        const piece = catalog.pieces.find((pc) => pc.id === p.piece_id);
        if (!piece) return null;
        const shape = piece.orientations[p.orientation];
        if (!shape) return null;
        const fill = shapeFill(piece.color, piece.special);
        const faulty = faultyIndices?.has(idx);
        const selected = selectedIndex === idx;
        return (
          <g
            key={idx}
            className={`piece-group${faulty ? " piece-faulty" : ""}${selected ? " piece-selected" : ""}`}
            onClick={onPieceClick ? () => onPieceClick(idx) : undefined}
            style={onPieceClick ? { cursor: "pointer" } : undefined}
          >
            {shape.map((cell, ci) => {
              const cx = ox + (p.anchor_x + cell.x) * CELL;
              const cy = oy + (p.anchor_y + cell.y) * CELL;
              if (cell.kind === "square") {
                return (
                  <rect key={ci} x={cx} y={cy} width={CELL} height={CELL} fill={fill} className="piece-shape" />
                );
              }
              const corner = cellKindCorner(cell.kind);
              const pts = TRIANGLE_POINTS[corner]
                .map(([ux, uy]) => `${cx + ux * CELL},${cy + uy * CELL}`)
                .join(" ");
              return <polygon key={ci} points={pts} fill={fill} className="piece-shape" />;
            })}
          </g>
        );
      })}

      {/* Marqueurs de sondage */}
      {markers?.map((m, i) => {
        const cx = ox + m.x * CELL + CELL / 2;
        const cy = oy + m.y * CELL + CELL / 2;
        const fill = m.kind === "color" && m.color ? GEM_COLORS[m.color] : m.kind === "occupied_no_color" ? "#333" : "none";
        return (
          <g key={i} className="probe-marker">
            <circle cx={cx} cy={cy} r={5} fill={fill} stroke="#1a1a1a" strokeWidth={1.5} />
          </g>
        );
      })}

      {/* Points de tir */}
      {points.map((pt) => {
        const px =
          pt.side === "left" ? ox - 12 : pt.side === "right" ? ox + w * CELL + 12 : ox + pt.cellX * CELL + CELL / 2;
        const py =
          pt.side === "top" ? oy - 12 : pt.side === "bottom" ? oy + h * CELL + 12 : oy + pt.cellY * CELL + CELL / 2;
        const used = usedPointIds?.has(pt.id);
        const role = highlightById.get(pt.id);
        return (
          <g
            key={pt.id}
            className={`board-point${used ? " board-point-used" : ""}${role ? ` board-point-${role}` : ""}`}
            onClick={onPointClick ? () => onPointClick(pt.id) : undefined}
            style={onPointClick ? { cursor: "pointer" } : undefined}
          >
            <circle cx={px} cy={py} r={9} />
            <text x={px} y={py} dy="0.32em" textAnchor="middle">
              {pt.id}
            </text>
          </g>
        );
      })}
    </svg>
  );
}

function cellKindCorner(kind: CellKind): "nw" | "ne" | "se" | "sw" {
  return kind.slice(4) as "nw" | "ne" | "se" | "sw";
}
