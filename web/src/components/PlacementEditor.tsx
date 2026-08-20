import { useEffect, useRef, useState } from "react";
import { Board, screenToCell } from "./Board";
import type { GhostPiece } from "./Board";
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

// Distance (en pixels écran) à parcourir avant qu'un appui sur une pièce
// (plateau ou plateau de pièces) ne soit considéré comme un glisser plutôt
// qu'un simple clic/tap de sélection.
const DRAG_THRESHOLD = 6;

interface DragState {
  source: "tray" | "board";
  pieceId: string;
  orientation: number;
  placedIndex: number | null;
  pointerId: number;
  grabDX: number;
  grabDY: number;
  moved: boolean;
  anchorX: number;
  anchorY: number;
  onBoard: boolean;
}

export function PlacementEditor({ catalog, pieces, onChange, diamond, black, onViolationsChange }: PlacementEditorProps) {
  const [pendingPieceId, setPendingPieceId] = useState<string | null>(null);
  const [selectedIndex, setSelectedIndex] = useState<number | null>(null);
  const [violations, setViolations] = useState<Violation[]>([]);
  const [drag, setDrag] = useState<DragState | null>(null);
  const [hoverCell, setHoverCell] = useState<{ x: number; y: number } | null>(null);

  const svgRef = useRef<SVGSVGElement>(null);
  const dragStartClient = useRef<{ x: number; y: number } | null>(null);
  const justDraggedRef = useRef(false);

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

  // Cases occupées par une pièce à une position/orientation donnée
  // (granularité case entière, sans distinguer les demi-cases triangulaires
  // — suffisant pour un indice visuel rapide pendant le glisser ; la
  // validation exacte reste faite par le wasm après pose, via l'effet
  // ci-dessus).
  function occupiedCells(pieceId: string, orientation: number, anchorX: number, anchorY: number): { x: number; y: number }[] {
    const piece = catalog.pieces.find((p) => p.id === pieceId);
    const shape = piece?.orientations[orientation];
    if (!shape) return [];
    const seen = new Map<string, { x: number; y: number }>();
    for (const c of shape) {
      const x = anchorX + c.x;
      const y = anchorY + c.y;
      seen.set(`${x},${y}`, { x, y });
    }
    return [...seen.values()];
  }

  function isGhostValid(pieceId: string, orientation: number, anchorX: number, anchorY: number, excludeIndex: number | null): boolean {
    const cells = occupiedCells(pieceId, orientation, anchorX, anchorY);
    if (cells.length === 0) return false;
    for (const c of cells) {
      if (c.x < 0 || c.y < 0 || c.x >= catalog.grid_width || c.y >= catalog.grid_height) return false;
    }
    const occupied = new Set<string>();
    pieces.forEach((p, i) => {
      if (i === excludeIndex) return;
      for (const c of occupiedCells(p.piece_id, p.orientation, p.anchor_x, p.anchor_y)) {
        occupied.add(`${c.x},${c.y}`);
      }
    });
    return cells.every((c) => !occupied.has(`${c.x},${c.y}`));
  }

  function placePending(x: number, y: number) {
    if (!pendingPieceId) return;
    const next = [...pieces, { piece_id: pendingPieceId, orientation: 0, anchor_x: x, anchor_y: y }];
    onChange(next);
    setPendingPieceId(null);
    setHoverCell(null);
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

  function nudgeSelected(dx: number, dy: number) {
    if (selectedIndex === null) return;
    const next = pieces.map((pp, i) =>
      i === selectedIndex ? { ...pp, anchor_x: pp.anchor_x + dx, anchor_y: pp.anchor_y + dy } : pp,
    );
    onChange(next);
  }

  // Raccourcis clavier une fois une pièce posée sélectionnée : pivoter (R),
  // retirer (Suppr/Retour arrière), déplacer d'une case (flèches) — une
  // alternative précise au glisser à la souris/au doigt.
  useEffect(() => {
    if (selectedIndex === null) return;
    function onKeyDown(e: KeyboardEvent) {
      if (e.key === "r" || e.key === "R") {
        rotateSelected();
        e.preventDefault();
      } else if (e.key === "Delete" || e.key === "Backspace") {
        removeSelected();
        e.preventDefault();
      } else if (e.key === "ArrowUp") {
        nudgeSelected(0, -1);
        e.preventDefault();
      } else if (e.key === "ArrowDown") {
        nudgeSelected(0, 1);
        e.preventDefault();
      } else if (e.key === "ArrowLeft") {
        nudgeSelected(-1, 0);
        e.preventDefault();
      } else if (e.key === "ArrowRight") {
        nudgeSelected(1, 0);
        e.preventDefault();
      } else if (e.key === "Escape") {
        setSelectedIndex(null);
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedIndex, pieces]);

  function beginTrayDrag(pieceId: string, e: React.PointerEvent) {
    if (pieces.some((p) => p.piece_id === pieceId)) return;
    e.currentTarget.setPointerCapture(e.pointerId);
    dragStartClient.current = { x: e.clientX, y: e.clientY };
    setSelectedIndex(null);
    setHoverCell(null);
    setDrag({
      source: "tray",
      pieceId,
      orientation: 0,
      placedIndex: null,
      pointerId: e.pointerId,
      grabDX: 0,
      grabDY: 0,
      moved: false,
      anchorX: -999,
      anchorY: -999,
      onBoard: false,
    });
  }

  function beginBoardDrag(index: number, e: React.PointerEvent) {
    const p = pieces[index];
    if (!p || !svgRef.current) return;
    e.currentTarget.setPointerCapture(e.pointerId);
    dragStartClient.current = { x: e.clientX, y: e.clientY };
    const cell = screenToCell(svgRef.current, e.clientX, e.clientY, catalog.grid_width, catalog.grid_height);
    setPendingPieceId(null);
    setDrag({
      source: "board",
      pieceId: p.piece_id,
      orientation: p.orientation,
      placedIndex: index,
      pointerId: e.pointerId,
      grabDX: cell.x - p.anchor_x,
      grabDY: cell.y - p.anchor_y,
      moved: false,
      anchorX: p.anchor_x,
      anchorY: p.anchor_y,
      onBoard: true,
    });
  }

  function onDragMove(e: React.PointerEvent) {
    if (!drag || !svgRef.current) return;
    const start = dragStartClient.current;
    const movedFar = start ? Math.hypot(e.clientX - start.x, e.clientY - start.y) > DRAG_THRESHOLD : false;
    const rect = svgRef.current.getBoundingClientRect();
    const withinRect = e.clientX >= rect.left && e.clientX <= rect.right && e.clientY >= rect.top && e.clientY <= rect.bottom;
    const cell = screenToCell(svgRef.current, e.clientX, e.clientY, catalog.grid_width, catalog.grid_height);
    setDrag({
      ...drag,
      moved: drag.moved || movedFar,
      anchorX: cell.x - drag.grabDX,
      anchorY: cell.y - drag.grabDY,
      onBoard: withinRect,
    });
  }

  function onDragEnd() {
    if (!drag) return;
    if (drag.moved) {
      justDraggedRef.current = true;
      if (drag.onBoard) {
        if (drag.source === "tray") {
          onChange([...pieces, { piece_id: drag.pieceId, orientation: 0, anchor_x: drag.anchorX, anchor_y: drag.anchorY }]);
        } else if (drag.placedIndex !== null) {
          const idx = drag.placedIndex;
          onChange(pieces.map((pp, i) => (i === idx ? { ...pp, anchor_x: drag.anchorX, anchor_y: drag.anchorY } : pp)));
        }
      }
    }
    setDrag(null);
  }

  function onDragCancel() {
    setDrag(null);
  }

  function handleTraySelect(id: string) {
    if (justDraggedRef.current) {
      justDraggedRef.current = false;
      return;
    }
    setSelectedIndex(null);
    setPendingPieceId((cur) => (cur === id ? null : id));
  }

  function handlePieceClick(index: number) {
    if (justDraggedRef.current) {
      justDraggedRef.current = false;
      return;
    }
    setPendingPieceId(null);
    setSelectedIndex((cur) => (cur === index ? null : index));
  }

  let ghost: GhostPiece | null = null;
  if (drag) {
    ghost = {
      pieceId: drag.pieceId,
      orientation: drag.orientation,
      anchorX: drag.anchorX,
      anchorY: drag.anchorY,
      valid: drag.onBoard && isGhostValid(drag.pieceId, drag.orientation, drag.anchorX, drag.anchorY, drag.placedIndex),
    };
  } else if (pendingPieceId && hoverCell) {
    ghost = {
      pieceId: pendingPieceId,
      orientation: 0,
      anchorX: hoverCell.x,
      anchorY: hoverCell.y,
      valid: isGhostValid(pendingPieceId, 0, hoverCell.x, hoverCell.y, null),
    };
  }

  return (
    <div className="placement-editor">
      <PieceTray
        catalog={catalog}
        placed={pieces}
        pendingPieceId={pendingPieceId}
        onSelect={handleTraySelect}
        onPointerDownPiece={beginTrayDrag}
        onDragPointerMove={onDragMove}
        onDragPointerUp={onDragEnd}
        onDragPointerCancel={onDragCancel}
        diamond={diamond}
        black={black}
      />
      <Board
        ref={svgRef}
        catalog={catalog}
        placements={pieces}
        onCellClick={pendingPieceId ? placePending : undefined}
        onCellHover={pendingPieceId && !drag ? (x, y) => setHoverCell({ x, y }) : undefined}
        onBoardLeave={() => setHoverCell(null)}
        onPiecePointerDown={beginBoardDrag}
        onPieceClick={handlePieceClick}
        onDragPointerMove={onDragMove}
        onDragPointerUp={onDragEnd}
        onDragPointerCancel={onDragCancel}
        selectedIndex={selectedIndex}
        draggingIndex={drag?.source === "board" ? drag.placedIndex : null}
        faultyIndices={faultyIndices}
        ghost={ghost}
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
