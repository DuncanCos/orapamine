import type { PieceCatalog, PlacedPiece } from "../types/protocol";
import { GEM_COLORS, SPECIAL_COLORS } from "../lib/colors";
import { t } from "../i18n/fr";

interface PieceTrayProps {
  catalog: PieceCatalog;
  placed: PlacedPiece[];
  pendingPieceId: string | null;
  onSelect: (pieceId: string) => void;
  onPointerDownPiece?: (pieceId: string, e: React.PointerEvent<HTMLButtonElement>) => void;
  onDragPointerMove?: (e: React.PointerEvent<HTMLButtonElement>) => void;
  onDragPointerUp?: (e: React.PointerEvent<HTMLButtonElement>) => void;
  onDragPointerCancel?: (e: React.PointerEvent<HTMLButtonElement>) => void;
  diamond: boolean;
  black: boolean;
}

const LABEL_KEYS: Record<string, string> = {
  red: "piece.red",
  yellow: "piece.yellow",
  blue: "piece.blue",
  white1: "piece.white1",
  white2: "piece.white2",
  diamond: "piece.diamond",
  black: "piece.black",
};

export function PieceTray({
  catalog,
  placed,
  pendingPieceId,
  onSelect,
  onPointerDownPiece,
  onDragPointerMove,
  onDragPointerUp,
  onDragPointerCancel,
  diamond,
  black,
}: PieceTrayProps) {
  const baseIds = ["red", "yellow", "blue", "white1", "white2"];
  const ids = [...baseIds, ...(diamond ? ["diamond"] : []), ...(black ? ["black"] : [])];
  const placedIds = new Set(placed.map((p) => p.piece_id));

  return (
    <div className="piece-tray">
      {ids.map((id) => {
        const piece = catalog.pieces.find((p) => p.id === id);
        if (!piece) return null;
        const isPlaced = placedIds.has(id);
        const isPending = pendingPieceId === id;
        const swatch = piece.color ? GEM_COLORS[piece.color] : piece.special === "absorb" ? SPECIAL_COLORS.absorb : SPECIAL_COLORS.transparent;
        return (
          <button
            key={id}
            type="button"
            className={`piece-tray-item${isPlaced ? " piece-tray-item-placed" : ""}${isPending ? " piece-tray-item-pending" : ""}`}
            onClick={() => onSelect(id)}
            onPointerDown={onPointerDownPiece ? (e) => onPointerDownPiece(id, e) : undefined}
            onPointerMove={onDragPointerMove}
            onPointerUp={onDragPointerUp}
            onPointerCancel={onDragPointerCancel}
            disabled={isPlaced}
          >
            <span className="piece-tray-swatch" style={{ background: swatch }} />
            {t(LABEL_KEYS[id] as never)}
          </button>
        );
      })}
    </div>
  );
}
