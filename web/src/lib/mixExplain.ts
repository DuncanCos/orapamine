import { RESULT_MIX } from "./colors";
import type { GemColor, ResultColor } from "../types/protocol";
import { t } from "../i18n/fr";

// Accord au féminin ("gemme bleue", pas "gemme bleu") pour la phrase
// d'explication — les libellés de `color.*` sont au masculin (adjectif seul).
const GEM_FEMININE: Record<GemColor, string> = {
  red: "rouge",
  blue: "bleue",
  yellow: "jaune",
  white: "blanche",
};

/** Phrase d'explication d'un résultat de mélange, pour les infobulles de la
 * légende et du plateau (ex. "Violet = Rouge + Bleu : l'onde a traversé une
 * gemme rouge et une gemme bleue."). */
export function mixExplanation(result: ResultColor): string {
  const gems = RESULT_MIX[result];
  const resultLabel = t(`color.${result}` as never);
  if (gems.length === 0) return `${resultLabel} : aucune gemme colorée traversée.`;
  const gemLabels = gems.map((g) => t(`color.${g}` as never));
  const combo = gemLabels.join(" + ");
  const feminineLabels = gems.map((g) => GEM_FEMININE[g]);
  const traversed =
    feminineLabels.length === 1
      ? `une gemme ${feminineLabels[0]}`
      : `les gemmes ${feminineLabels.slice(0, -1).join(", ")} et ${feminineLabels[feminineLabels.length - 1]}`;
  return `${combo} = ${resultLabel} : l'onde a traversé ${traversed}.`;
}
