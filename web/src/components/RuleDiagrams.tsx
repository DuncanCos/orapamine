import { GEM_COLORS, RESULT_COLORS } from "../lib/colors";
import { t } from "../i18n/fr";

// Petits schémas SVG autonomes (pas de dépendance à Board/catalog) pour
// illustrer les règles les plus difficiles à saisir par le seul texte —
// même charte pixel-art que le plateau (traits épais, teintes plates).

const CELL = 34;
const STROKE = "#1a1712";

function Cell({ x, y, fill }: { x: number; y: number; fill: string }) {
  return <rect x={x * CELL} y={y * CELL} width={CELL} height={CELL} fill={fill} stroke={STROKE} strokeWidth={2} />;
}

function Grid({ cols, rows }: { cols: number; rows: number }) {
  const lines = [];
  for (let i = 0; i <= cols; i++) {
    lines.push(<line key={`v${i}`} x1={i * CELL} y1={0} x2={i * CELL} y2={rows * CELL} stroke="#e4d8bd" />);
  }
  for (let j = 0; j <= rows; j++) {
    lines.push(<line key={`h${j}`} x1={0} y1={j * CELL} x2={cols * CELL} y2={j * CELL} stroke="#e4d8bd" />);
  }
  return <>{lines}</>;
}

function Mark({ x, y, ok }: { x: number; y: number; ok: boolean }) {
  const cx = x * CELL;
  const cy = y * CELL;
  return ok ? (
    <path
      d={`M ${cx - 9} ${cy} L ${cx - 2} ${cy + 8} L ${cx + 10} ${cy - 9}`}
      fill="none"
      stroke="#4fae5b"
      strokeWidth={4}
      strokeLinecap="round"
      strokeLinejoin="round"
    />
  ) : (
    <g stroke="#d3392f" strokeWidth={4} strokeLinecap="round">
      <line x1={cx - 8} y1={cy - 8} x2={cx + 8} y2={cy + 8} />
      <line x1={cx + 8} y1={cy - 8} x2={cx - 8} y2={cy + 8} />
    </g>
  );
}

/** Illustre la règle de contact entre pièces : coin-à-coin autorisé,
 * arête-à-arête interdit. */
export function PlacementDiagram() {
  return (
    <div className="rules-diagram rules-diagram-pair">
      <figure>
        <svg viewBox={`0 0 ${CELL * 3} ${CELL * 2 + 20}`} className="rules-diagram-svg">
          <g transform="translate(0, 10)">
            <Grid cols={3} rows={2} />
            <Cell x={0} y={0} fill={GEM_COLORS.red} />
            <Cell x={1} y={1} fill={GEM_COLORS.blue} />
            <Mark x={1} y={1} ok />
          </g>
        </svg>
        <figcaption>{t("rules.diagram.corner_ok")}</figcaption>
      </figure>
      <figure>
        <svg viewBox={`0 0 ${CELL * 3} ${CELL * 2 + 20}`} className="rules-diagram-svg">
          <g transform="translate(0, 10)">
            <Grid cols={3} rows={2} />
            <Cell x={0} y={0} fill={GEM_COLORS.red} />
            <Cell x={1} y={0} fill={GEM_COLORS.blue} />
            <Mark x={1} y={0.5} ok={false} />
          </g>
        </svg>
        <figcaption>{t("rules.diagram.edge_bad")}</figcaption>
      </figure>
    </div>
  );
}

/** Illustre les deux comportements du faisceau : demi-tour sur une face
 * droite (carré), déviation à 90° sur l'hypoténuse d'un triangle. */
export function BeamDiagram() {
  const arrow = (id: string, color: string) => (
    <marker id={id} viewBox="0 0 10 10" refX="6" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
      <path d="M 0 0 L 10 5 L 0 10 z" fill={color} />
    </marker>
  );
  return (
    <div className="rules-diagram rules-diagram-pair">
      <figure>
        <svg viewBox={`0 0 ${CELL * 3} ${CELL * 3}`} className="rules-diagram-svg">
          <defs>{arrow("arrow-bounce", "#2f6fd3")}</defs>
          <Grid cols={3} rows={3} />
          <Cell x={1} y={1} fill={GEM_COLORS.red} />
          <path
            d={`M 0 ${1.5 * CELL} H ${1 * CELL} M ${1 * CELL} ${1.5 * CELL + 6} H 0`}
            fill="none"
            stroke="#2f6fd3"
            strokeWidth={3}
            markerEnd="url(#arrow-bounce)"
            transform={`translate(0,-3)`}
          />
        </svg>
        <figcaption>{t("rules.diagram.bounce")}</figcaption>
      </figure>
      <figure>
        <svg viewBox={`0 0 ${CELL * 3} ${CELL * 3}`} className="rules-diagram-svg">
          <defs>{arrow("arrow-deflect", "#2f6fd3")}</defs>
          <Grid cols={3} rows={3} />
          <polygon
            points={`${1 * CELL},${1 * CELL} ${2 * CELL},${1 * CELL} ${1 * CELL},${2 * CELL}`}
            fill={GEM_COLORS.blue}
            stroke={STROKE}
            strokeWidth={2}
          />
          <path
            d={`M ${1.5 * CELL} 0 V ${1 * CELL} H ${3 * CELL}`}
            fill="none"
            stroke="#2f6fd3"
            strokeWidth={3}
            markerEnd="url(#arrow-deflect)"
          />
        </svg>
        <figcaption>{t("rules.diagram.deflect")}</figcaption>
      </figure>
    </div>
  );
}

/** Illustre le mélange des couleurs : une onde qui traverse une gemme
 * rouge puis une gemme blanche ressort rose. */
export function MixDiagram() {
  return (
    <div className="rules-diagram">
      <svg viewBox={`0 0 ${CELL * 5} ${CELL * 1.6}`} className="rules-diagram-svg rules-diagram-wide">
        <g transform={`translate(0, ${CELL * 0.3})`}>
          <rect width={CELL} height={CELL} fill={GEM_COLORS.red} stroke={STROKE} strokeWidth={2} />
          <text x={CELL * 1.5} y={CELL * 0.65} textAnchor="middle" className="rules-diagram-plus">
            +
          </text>
          <rect x={CELL * 2} width={CELL} height={CELL} fill={GEM_COLORS.white} stroke={STROKE} strokeWidth={2} />
          <text x={CELL * 3.5} y={CELL * 0.65} textAnchor="middle" className="rules-diagram-plus">
            →
          </text>
          <rect x={CELL * 4} width={CELL} height={CELL} fill={RESULT_COLORS.pink} stroke={STROKE} strokeWidth={2} />
        </g>
      </svg>
    </div>
  );
}
