// i18n minimaliste : un seul dictionnaire plat + un helper `t()`. Pas de
// librairie — le jeu n'a qu'une langue pour l'instant, mais tous les
// libellés sont centralisés ici pour en ajouter une facilement plus tard.

const fr = {
  "app.title": "Orapa Mine",
  "app.leave_game": "Retour au menu",
  "app.leave_confirm": "Quitter la partie en cours et revenir au menu principal ?",
  "lobby.title": "Orapa Mine",
  "lobby.pseudo": "Pseudo",
  "lobby.pseudo.placeholder": "Ton nom de joueur",
  "lobby.pseudo.random": "Nouveau pseudo aléatoire",
  "lobby.create": "Créer une partie",
  "lobby.join": "Rejoindre une partie",
  "lobby.code": "Code de la partie",
  "lobby.code.placeholder": "Ex. ABCDE",
  "lobby.solo": "Jouer en solo (entraînement)",
  "lobby.options": "Options",
  "lobby.options.diamond": "Extension Diamant",
  "lobby.options.black": "Extension Corps noir",
  "lobby.options.lives": "Vies avant élimination",
  "lobby.options.timer": "Limite de temps par tour",
  "lobby.options.timer.none": "Aucune",
  "lobby.options.help_mode": "Mode aide (vérifie tes hypothèses)",
  "lobby.error.pseudo_required": "Choisis un pseudo avant de continuer.",
  "lobby.error.code_required": "Entre le code de la partie à rejoindre.",
  "lobby.connecting": "Connexion…",
  "lobby.share_code": "Code à partager : ",
  "lobby.waiting_opponent": "En attente d'un adversaire…",

  "placement.title": "Place tes gemmes",
  "placement.instructions":
    "Fais glisser une pièce vers la grille pour la poser, ou clique dessus puis sur une case. Fais glisser une pièce posée pour la déplacer, ou clique dessus pour la faire pivoter (touche R), la déplacer aux flèches, ou la retirer (Suppr).",
  "placement.rotate": "Pivoter",
  "placement.mirror": "Miroir",
  "placement.remove": "Retirer",
  "placement.random": "Placement aléatoire valide",
  "placement.validate": "Valider mon placement",
  "placement.ready": "Je suis prêt",
  "placement.waiting_ready": "En attente que ton adversaire valide…",
  "placement.violations.title": "Placement invalide :",
  "placement.violations.out_of_bounds": "Une pièce sort de la grille.",
  "placement.violations.overlap": "Deux pièces se chevauchent.",
  "placement.violations.edge_contact": "Deux pièces se touchent par une arête pleine.",
  "placement.violations.unreachable": "Une pièce n'est atteignable par aucune onde.",
  "placement.violations.white_symmetry": "Les deux gemmes blanches sont placées symétriquement.",

  "game.your_turn": "À toi de jouer",
  "game.opponent_turn": "Tour de l'adversaire",
  "game.lives": "Vies",
  "game.action.beam": "Envoyer une onde",
  "game.action.probe": "Sonder une case",
  "game.action.solution": "Proposer une solution",
  "game.action.pick_point": "Clique sur un point du bord de la grille adverse.",
  "game.action.pick_cell": "Clique sur une case de la grille adverse.",
  "game.beam.click_hint": "Clique sur un point déjà tiré pour mettre son trajet en évidence.",
  "game.action.submit_hint":
    "Construis ta grille d'hypothèses ci-dessous puis valide pour la soumettre comme solution.",
  "game.hypothesis.title": "Grille d'hypothèses",
  "game.hypothesis.check": "Vérifier la cohérence",
  "game.hypothesis.consistent": "Cohérent avec tous les indices reçus.",
  "game.hypothesis.inconsistent": "Contredit certains indices :",
  "game.history.title": "Historique",
  "game.history.empty": "Aucune action pour l'instant.",
  "game.history.beam": "onde",
  "game.history.probe": "sondage",
  "game.history.timeout": "temps écoulé",
  "game.legend.title": "Mélange des couleurs",
  "game.sudden_death": "Mort subite : dernier tour pour égaliser !",
  "game.reactions.gg": "Bien joué !",
  "game.reactions.oops": "Aïe !",
  "game.reactions.think": "Je réfléchis…",
  "game.solution.reveal": "Voir la solution",
  "game.solution.title": "Solution",

  "result.title": "Partie terminée",
  "result.win": "Tu as gagné !",
  "result.lose": "Tu as perdu.",
  "result.draw": "Égalité !",
  "result.rematch": "Revanche",
  "result.waiting_rematch": "En attente de la revanche…",
  "result.boards.yours": "Ta grille",
  "result.boards.opponent": "Grille adverse",

  "color.transparent": "Transparent",
  "color.red": "Rouge",
  "color.blue": "Bleu",
  "color.yellow": "Jaune",
  "color.white": "Blanc",
  "color.purple": "Violet",
  "color.orange": "Orange",
  "color.green": "Vert",
  "color.pink": "Rose",
  "color.sky_blue": "Bleu ciel",
  "color.lemon": "Citron",
  "color.black": "Noir",
  "color.light_purple": "Violet clair",
  "color.light_green": "Vert clair",
  "color.light_orange": "Orange clair",
  "color.gray": "Gris",
  "color.absorbed": "Signal absorbé",
  "color.lost": "Onde perdue",

  "piece.red": "Rouge",
  "piece.yellow": "Jaune",
  "piece.blue": "Bleu",
  "piece.white1": "Blanc (grand triangle)",
  "piece.white2": "Blanc (losange)",
  "piece.diamond": "Diamant",
  "piece.black": "Corps noir",

  "conn.disconnected": "Connexion perdue, tentative de reconnexion…",
  "conn.reconnected": "Reconnecté.",

  "rules.link": "Comment jouer ?",
  "rules.title": "Comment jouer",
  "rules.close": "Fermer",
  "rules.goal.title": "Le but du jeu",
  "rules.goal.body":
    "Chaque joueur cache secrètement 5 gemmes (formes tangram) sur sa grille 10×8. À ton tour, tu sondes la grille de ton adversaire en tirant des ondes lumineuses depuis le pourtour pour déduire où se cachent ses gemmes. Le premier à trouver la disposition complète de l'adversaire gagne — mais gare aux fausses pistes, une proposition erronée te coûte une vie.",
  "rules.setup.title": "Mise en place",
  "rules.setup.body":
    "Place tes 5 gemmes (1 rouge, 1 jaune, 1 bleue, 2 blanches) sur ta grille : glisse une pièce depuis la réserve, clique dessus pour la faire pivoter ou la retourner, sans qu'elle sorte de la grille, chevauche une autre pièce ou la touche par une arête pleine (les coins peuvent se toucher). Chaque gemme doit rester atteignable par au moins une onde, et les deux blanches ne peuvent pas être placées en symétrie parfaite.",
  "rules.turn.title": "À ton tour",
  "rules.turn.beam":
    "Envoyer une onde : clique sur un point du pourtour de la grille adverse. L'onde se propage en ligne droite, rebondit ou dévie en touchant une gemme, puis ressort quelque part sur le pourtour avec une couleur qui dépend des gemmes traversées.",
  "rules.turn.probe":
    "Sonder une case : clique sur une case précise de la grille adverse pour savoir si une gemme s'y trouve.",
  "rules.turn.solution":
    "Proposer une solution : place tes hypothèses sur la grille d'hypothèses puis valide. Si tu te trompes, tu perds une vie (1 ou 2 selon les options de la partie) ; à 0 vie, tu es éliminé.",
  "rules.beam.title": "Lire le résultat d'une onde",
  "rules.beam.body":
    "Une onde qui touche une face droite d'une gemme fait demi-tour ; une onde qui touche l'hypoténuse d'un triangle dévie à 90°. Chaque couleur de gemme traversée s'ajoute au mélange final (voir le tableau ci-dessous) — une onde qui ne touche rien ressort transparente.",
  "rules.end.title": "Fin de partie",
  "rules.end.body":
    "Une proposition juste gagne la partie immédiatement — sauf si tu es le premier joueur du tour à trouver : ton adversaire a alors droit à un dernier tour pour égaliser (mort subite). En mode solo, entraîne-toi à déduire une disposition tirée au hasard, sans adversaire.",

  "rules.diagram.corner_ok": "Coin-à-coin : autorisé",
  "rules.diagram.edge_bad": "Arête-à-arête : interdit",
  "rules.diagram.bounce": "Face droite → demi-tour",
  "rules.diagram.deflect": "Hypoténuse → déviation à 90°",
} as const;

export type TranslationKey = keyof typeof fr;

export function t(key: TranslationKey): string {
  return fr[key];
}
