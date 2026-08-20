import { ColorLegend } from "./ColorLegend";
import { t } from "../i18n/fr";

export function RulesModal({ onClose }: { onClose: () => void }) {
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-box rules-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>{t("rules.title")}</h2>
          <button type="button" className="modal-close" onClick={onClose} aria-label={t("rules.close")}>
            ✕
          </button>
        </div>

        <section>
          <h3>{t("rules.goal.title")}</h3>
          <p>{t("rules.goal.body")}</p>
        </section>

        <section>
          <h3>{t("rules.setup.title")}</h3>
          <p>{t("rules.setup.body")}</p>
        </section>

        <section>
          <h3>{t("rules.turn.title")}</h3>
          <p>{t("rules.turn.beam")}</p>
          <p>{t("rules.turn.probe")}</p>
          <p>{t("rules.turn.solution")}</p>
        </section>

        <section>
          <h3>{t("rules.beam.title")}</h3>
          <p>{t("rules.beam.body")}</p>
          <ColorLegend />
        </section>

        <section>
          <h3>{t("rules.end.title")}</h3>
          <p>{t("rules.end.body")}</p>
        </section>

        <button type="button" className="primary rules-close-button" onClick={onClose}>
          {t("rules.close")}
        </button>
      </div>
    </div>
  );
}
