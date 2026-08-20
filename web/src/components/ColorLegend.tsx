import { RESULT_COLORS } from "../lib/colors";
import { mixExplanation } from "../lib/mixExplain";
import type { ResultColor } from "../types/protocol";
import { t } from "../i18n/fr";

const ROWS: { colors: string; result: ResultColor }[] = [
  { colors: "—", result: "transparent" },
  { colors: t("color.red"), result: "red" },
  { colors: t("color.blue"), result: "blue" },
  { colors: t("color.yellow"), result: "yellow" },
  { colors: t("color.white"), result: "white" },
  { colors: `${t("color.red")} + ${t("color.blue")}`, result: "purple" },
  { colors: `${t("color.red")} + ${t("color.yellow")}`, result: "orange" },
  { colors: `${t("color.blue")} + ${t("color.yellow")}`, result: "green" },
  { colors: `${t("color.red")} + ${t("color.white")}`, result: "pink" },
  { colors: `${t("color.blue")} + ${t("color.white")}`, result: "sky_blue" },
  { colors: `${t("color.yellow")} + ${t("color.white")}`, result: "lemon" },
  { colors: `${t("color.red")} + ${t("color.blue")} + ${t("color.yellow")}`, result: "black" },
  { colors: `${t("color.red")} + ${t("color.blue")} + ${t("color.white")}`, result: "light_purple" },
  { colors: `${t("color.blue")} + ${t("color.yellow")} + ${t("color.white")}`, result: "light_green" },
  { colors: `${t("color.red")} + ${t("color.yellow")} + ${t("color.white")}`, result: "light_orange" },
  { colors: `Rouge + Bleu + Jaune + Blanc`, result: "gray" },
];

interface ColorLegendProps {
  /** Résultat actuellement survolé/épinglé sur le plateau — met en évidence
   * la ligne correspondante pour relier immédiatement une onde tirée à ce
   * qu'elle implique, sans avoir à recomparer la couleur "à l'œil". */
  highlightedResult?: ResultColor | null;
}

export function ColorLegend({ highlightedResult }: ColorLegendProps) {
  return (
    <details className="color-legend" open>
      <summary>{t("game.legend.title")}</summary>
      <table>
        <tbody>
          {ROWS.map((row) => (
            <tr
              key={row.result}
              className={row.result === highlightedResult ? "legend-row-highlight" : undefined}
              title={mixExplanation(row.result)}
            >
              <td>{row.colors}</td>
              <td>
                <span className="legend-swatch" style={{ background: RESULT_COLORS[row.result] }} />
                {t(`color.${row.result}` as never)}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </details>
  );
}
