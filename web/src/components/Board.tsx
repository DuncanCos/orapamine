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

/** Trajet complet d'une onde (point d'entrée -> point de sortie), pour
 * dessiner une ligne directement sur le plateau plutôt que de forcer à
 * aller consulter l'historique. Les ondes absorbées ou perdues n'ont pas
 * de `toId` : on trace un petit segment depuis le point d'entrée vers le
 * centre de la grille pour indiquer "entrée sans ressortir". */
export interface BeamLine {
  id: string;
  fromId: string;
  toId: string | null;
  color: string;
  kind: "exit" | "absorbed" | "lost";
}

export interface BoardProps {
  catalog: PieceCatalog;
  placements: PlacedPiece[];
  showPoints?: boolean;
  usedPointIds?: Set<string>;
  onPointClick?: (id: string) => void;
  onPointHover?: (id: string | null) => void;
  onCellClick?: (x: number, y: number) => void;
  onPieceClick?: (index: number) => void;
  selectedIndex?: number | null;
  faultyIndices?: Set<number>;
  markers?: ProbeMarker[];
  highlightPoints?: PointHighlight[];
  beamLines?: BeamLine[];
  emphasizedLineId?: string | null;
}

export function Board({
  catalog,
  placements,
  showPoints = false,
  usedPointIds,
  onPointClick,
  onPointHover,
  onCellClick,
  onPieceClick,
  selectedIndex = null,
  faultyIndices,
  markers,
  highlightPoints,
  beamLines,
  emphasizedLineId,
}: BoardProps) {
  const w = catalog.grid_width;
  const h = catalog.grid_height;
  const ox = MARGIN;
  const oy = MARGIN;
  const viewW = ox * 2 + w * CELL;
  const viewH = oy * 2 + h * CELL;
  const points = showPoints ? boardPoints(w, h) : [];
  const highlightById = new Map((highlightPoints ?? []).map((p) => [p.id, p.role]));
  const pointById = new Map(points.map((pt) => [pt.id, pt]));
  const centerX = ox + (w * CELL) / 2;
  const centerY = oy + (h * CELL) / 2;

  function pointPos(id: string): { x: number; y: number } | null {
    const pt = pointById.get(id);
    if (!pt) return null;
    const x = pt.side === "left" ? ox - 12 : pt.side === "right" ? ox + w * CELL + 12 : ox + pt.cellX * CELL + CELL / 2;
    const y = pt.side === "top" ? oy - 12 : pt.side === "bottom" ? oy + h * CELL + 12 : oy + pt.cellY * CELL + CELL / 2;
    return { x, y };
  }

  // L'onde émphasisée se dessine en dernier (par-dessus les autres).
  const orderedLines = (beamLines ?? []).slice().sort((a, b) => {
    if (a.id === emphasizedLineId) return 1;
    if (b.id === emphasizedLineId) return -1;
    return 0;
  });

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

      {/* Trajets des ondes déjà tirées : ligne directe entrée -> sortie,
          couleur du résultat final, pour ne pas avoir à consulter
          l'historique. Émphasisée (survol/clic) = plus épaisse, par-dessus. */}
      {orderedLines.map((line) => {
        const from = pointPos(line.fromId);
        if (!from) return null;
        const emphasized = line.id === emphasizedLineId;
        if (line.kind === "exit" && line.toId && line.toId !== line.fromId) {
          const to = pointPos(line.toId);
          if (!to) return null;
          return (
            <line
              key={line.id}
              x1={from.x}
              y1={from.y}
              x2={to.x}
              y2={to.y}
              stroke={line.color}
              strokeWidth={emphasized ? 4 : 2.25}
              strokeOpacity={emphasized ? 1 : 0.6}
              strokeLinecap="round"
              className="beam-line"
            />
          );
        }
        // Absorbée / perdue, ou rebond immédiat (le point de sortie est le
        // point d'entrée lui-même, donc une "ligne" entre les deux serait
        // invisible) : petit segment vers le centre de la grille pour
        // indiquer "entrée sans en ressortir ailleurs".
        const dx = centerX - from.x;
        const dy = centerY - from.y;
        const len = Math.hypot(dx, dy) || 1;
        const stubLen = 22;
        const stubX = from.x + (dx / len) * stubLen;
        const stubY = from.y + (dy / len) * stubLen;
        return (
          <line
            key={line.id}
            x1={from.x}
            y1={from.y}
            x2={stubX}
            y2={stubY}
            stroke={line.color}
            strokeWidth={emphasized ? 4 : 2.25}
            strokeOpacity={emphasized ? 1 : 0.6}
            strokeLinecap="round"
            strokeDasharray={line.kind === "lost" ? "3 3" : undefined}
            className="beam-line"
          />
        );
      })}

      {/* Points de tir */}
      {points.map((pt) => {
        const { x: px, y: py } = pointPos(pt.id)!;
        const used = usedPointIds?.has(pt.id);
        const role = highlightById.get(pt.id);
        const emphasized = beamLines?.some((l) => l.id === emphasizedLineId && l.fromId === pt.id);
        return (
          <g
            key={pt.id}
            className={`board-point${used ? " board-point-used" : ""}${role ? ` board-point-${role}` : ""}${emphasized ? " board-point-emphasized" : ""}`}
            onClick={onPointClick ? () => onPointClick(pt.id) : undefined}
            onMouseEnter={onPointHover ? () => onPointHover(pt.id) : undefined}
            onMouseLeave={onPointHover ? () => onPointHover(null) : undefined}
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
