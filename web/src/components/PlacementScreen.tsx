import { useState } from "react";
import { useGameStore } from "../store/gameStore";
import { PlacementEditor } from "./PlacementEditor";
import type { PieceCatalog, Violation } from "../types/protocol";
import { t } from "../i18n/fr";

interface PlacementScreenProps {
  catalog: PieceCatalog;
}

export function PlacementScreen({ catalog }: PlacementScreenProps) {
  const ownPlacement = useGameStore((s) => s.ownPlacement);
  const setOwnPlacement = useGameStore((s) => s.setOwnPlacement);
  const requestRandomPlacement = useGameStore((s) => s.requestRandomPlacement);
  const submitPlacement = useGameStore((s) => s.submitPlacement);
  const readyToPlay = useGameStore((s) => s.readyToPlay);
  const options = useGameStore((s) => s.options);
  const players = useGameStore((s) => s.players);
  const yourIndex = useGameStore((s) => s.yourIndex);
  const serverViolations = useGameStore((s) => s.violations);

  const [localViolations, setLocalViolations] = useState<Violation[]>([]);

  const you = yourIndex !== null ? players[yourIndex] : undefined;
  const complete = ownPlacement.length === 5 + (options?.diamond ? 1 : 0) + (options?.black ? 1 : 0);
  const canValidate = complete && localViolations.length === 0;

  return (
    <div className="placement-screen">
      <h2>{t("placement.title")}</h2>
      <p className="muted">{t("placement.instructions")}</p>

      <PlacementEditor
        catalog={catalog}
        pieces={ownPlacement}
        onChange={setOwnPlacement}
        diamond={options?.diamond ?? false}
        black={options?.black ?? false}
        onViolationsChange={setLocalViolations}
      />

      {serverViolations.length > 0 && (
        <div className="violations-box violations-box-server">
          <strong>{t("placement.violations.title")}</strong>
          <ul>
            {serverViolations.map((v, i) => (
              <li key={i}>{v.kind}</li>
            ))}
          </ul>
        </div>
      )}

      <div className="placement-actions">
        <button type="button" onClick={requestRandomPlacement}>
          {t("placement.random")}
        </button>
        <button type="button" className="primary" disabled={!canValidate} onClick={submitPlacement}>
          {t("placement.validate")}
        </button>
        <button type="button" disabled={!you?.has_placement} onClick={readyToPlay}>
          {t("placement.ready")}
        </button>
      </div>

      {you?.ready && <p className="muted">{t("placement.waiting_ready")}</p>}
    </div>
  );
}
