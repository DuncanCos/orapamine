import { useEffect, useState } from "react";
import { Board } from "./Board";
import { PieceTray } from "./PieceTray";
import { validatePlacement } from "../wasmClient";
import type { PieceCatalog, PlacedPiece, Violation } from "../types/protocol";
import { t } from "../i18n/fr";

interface PlacementEditorProps {
  catalog: PieceCatalog;
  pieces: PlacedPiece[];
  onChange: (pieces: PlacedPiece[]) => void;
  diamond: boolean;
  black: boolean;
  onViolationsChange?: (violations: Violation[]) => void;
}

const VIOLATION_LABEL_KEYS: Record<Violation["kind"], string> = {
  OutOfBounds: "placement.violations.out_of_bounds",
  Overlap: "placement.violations.overlap",
  EdgeContact: "placement.violations.edge_contact",
  Unreachable: "placement.violations.unreachable",
  WhiteSymmetry: "placement.violations.white_symmetry",
};

export function PlacementEditor({ catalog, pieces, onChange, diamond, black, onViolationsChange }: PlacementEditorProps) {
  const [pendingPieceId, setPendingPieceId] = useState<string | null>(null);
  const [selectedIndex, setSelectedIndex] = useState<number | null>(null);
  const [violations, setViolations] = useState<Violation[]>([]);

  useEffect(() => {
    const v = validatePlacement(pieces);
    setViolations(v);
    onViolationsChange?.(v);
    // onViolationsChange volontairement omis des deps : identité de
    // fonction instable côté appelant, on ne veut relancer que sur
    // `pieces`.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pieces]);

  const faultyIndices = new Set<number>();
  for (const v of violations) {
    if ("placement_index" in v) faultyIndices.add(v.placement_index);
    if ("other_index" in v) faultyIndices.add(v.other_index);
  }

  function placePending(x: number, y: number) {
    if (!pendingPieceId) return;
    const next = [...pieces, { piece_id: pendingPieceId, orientation: 0, anchor_x: x, anchor_y: y }];
    onChange(next);
    setPendingPieceId(null);
  }

  function rotateSelected() {
    if (selectedIndex === null) return;
    const p = pieces[selectedIndex];
    const piece = catalog.pieces.find((pc) => pc.id === p.piece_id);
    if (!piece) return;
    const next = pieces.map((pp, i) =>
      i === selectedIndex ? { ...pp, orientation: (pp.orientation + 1) % piece.orientations.length } : pp,
    );
    onChange(next);
  }

  function removeSelected() {
    if (selectedIndex === null) return;
    onChange(pieces.filter((_, i) => i !== selectedIndex));
    setSelectedIndex(null);
  }

  return (
    <div className="placement-editor">
      <PieceTray
        catalog={catalog}
        placed={pieces}
        pendingPieceId={pendingPieceId}
        onSelect={(id) => {
          setSelectedIndex(null);
          setPendingPieceId((cur) => (cur === id ? null : id));
        }}
        diamond={diamond}
        black={black}
      />
      <Board
        catalog={catalog}
        placements={pieces}
        onCellClick={pendingPieceId ? placePending : undefined}
        onPieceClick={(idx) => {
          setPendingPieceId(null);
          setSelectedIndex((cur) => (cur === idx ? null : idx));
        }}
        selectedIndex={selectedIndex}
        faultyIndices={faultyIndices}
      />
      {selectedIndex !== null && (
        <div className="placement-editor-controls">
          <button type="button" onClick={rotateSelected}>
            {t("placement.rotate")}
          </button>
          <button type="button" onClick={removeSelected}>
            {t("placement.remove")}
          </button>
        </div>
      )}
      {violations.length > 0 && (
        <div className="violations-box">
          <strong>{t("placement.violations.title")}</strong>
          <ul>
            {violations.map((v, i) => (
              <li key={i}>{t(VIOLATION_LABEL_KEYS[v.kind] as never)}</li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
