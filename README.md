# Orapa Mine — en ligne

Adaptation web à 2 joueurs (+ mode solo d'entraînement) du jeu de déduction
**Orapa Mine** (Junghee Choi & Wanjin Gill, Playte 2024 / Miraludo 2026).
Chaque joueur cache 5 gemmes tangram sur une grille 10×8 ; l'adversaire
envoie des ondes depuis le pourtour pour en déduire la disposition à partir
du point de sortie et de la couleur du faisceau.

## Sommaire

- [Lancer en local](#lancer-en-local)
- [Déployer avec Docker](#déployer-avec-docker)
- [Architecture](#architecture)
- [Règles résumées](#règles-résumées)
- [Configurer les pièces (`data/pieces.json`)](#configurer-les-pièces-datapiecesjson)
- [Cas limites du faisceau](#cas-limites-du-faisceau)
- [Tests](#tests)
- [Limites connues](#limites-connues)

## Lancer en local

### Prérequis

- Rust stable (`rustup`), avec la cible wasm : `rustup target add wasm32-unknown-unknown`
- [`wasm-bindgen-cli`](https://github.com/rustwasm/wasm-bindgen), **version identique**
  à la dépendance `wasm-bindgen` du workspace (voir `Cargo.lock`) :
  ```sh
  cargo install wasm-bindgen-cli --version 0.2.127 --locked
  ```
- Node.js 22+ et npm

### Moteur de jeu + démo CLI

```sh
cargo test --workspace          # tous les tests (moteur, serveur, wasm)
cargo run -p orapa-cli -- --seed 1 --all      # trace les 36 ondes sur un plateau tiré au hasard
cargo run -p orapa-cli -- --seed 1 --beam 7   # une seule onde
cargo run -p orapa-cli -- --seed 1 --diamond --black --all   # avec les extensions
```

### Client + serveur en développement

```sh
# 1. Génère les bindings WASM consommés par le client (à refaire après
#    toute modification de crates/orapa-core ou crates/orapa-wasm)
cd web && npm install && npm run build:wasm

# 2. Le serveur (sert aussi les fichiers statiques du build de prod si
#    présents, mais en dev on utilise `npm run dev` côté client)
cd .. && cargo run -p orapa-server        # écoute sur :8080

# 3. Le client, avec rechargement à chaud (dans un autre terminal)
cd web && npm run dev                     # http://localhost:5173, proxy /ws -> :8080
```

> En dev, le client tourne sur le port de Vite (5173) tandis que le
> WebSocket cible `/ws` sur l'origine courante — pense à lancer les deux.
> Pour un test en conditions de production (un seul port, un seul
> processus), utilise plutôt `npm run build` côté client puis
> `ORAPA_STATIC_DIR=web/dist cargo run -p orapa-server`.

## Déployer avec Docker

```sh
docker compose up --build
```

C'est tout : une seule image multi-étapes (moteur → WASM, client Vite,
serveur natif, puis image d'exécution `debian:bookworm-slim` d'une
trentaine de Mo) sert le tout sur `http://localhost:8080`. Pas de base de
données, pas de service additionnel — les parties vivent en mémoire dans le
processus serveur (voir [Limites connues](#limites-connues)).

Variables d'environnement du service :

| Variable            | Défaut       | Rôle                                    |
|---------------------|--------------|------------------------------------------|
| `PORT`              | `8080`       | Port d'écoute HTTP + WebSocket           |
| `ORAPA_STATIC_DIR`  | `web/dist`   | Dossier des fichiers statiques du client |
| `RUST_LOG`          | *(aucun)*    | Niveau de log (`info`, `debug`, ...)     |

## Architecture

```
crates/
  orapa-core/    moteur pur (géométrie, faisceau, couleurs, placement, solution) — sans dépendance UI/réseau
  orapa-wasm/    bindings wasm-bindgen du moteur, pour le client
  orapa-cli/     démo ASCII du moteur
  orapa-server/  serveur axum : machine à états par partie, protocole WebSocket
data/
  pieces.json    catalogue des pièces (voir plus bas)
web/             client React + TypeScript (Vite)
```

Le moteur (`orapa-core`) est la seule implémentation de la géométrie, du
tracé de faisceau, de la validation de placement et du mélange des
couleurs — utilisée telle quelle côté serveur (Rust natif) et côté client
(compilée en WebAssembly). Les dispositions secrètes ne sont **jamais**
envoyées à l'adversaire avant la fin de partie ; tout calcul de faisceau,
sondage ou vérification de solution est fait côté serveur.

## Règles résumées

- **Plateau** : grille 10×8, 36 points de tir sur le pourtour (lettres A–R
  à gauche puis en bas, chiffres 1–18 en haut puis à droite).
- **Gemmes** : 1 rouge, 1 jaune, 1 bleue, 2 blanches (+ Diamant et Corps
  noir en extensions optionnelles). Formes tangram : cases pleines et
  demi-cases triangulaires, rotables et retournables.
- **Placement** : pas de chevauchement, pas de contact arête-à-arête entre
  deux pièces (coin-à-coin ou coin-à-arête autorisés), chaque pièce doit
  être atteignable par au moins une onde, les deux blanches ne peuvent pas
  être disposées en symétrie parfaite (verticale ou horizontale).
- **Faisceau** : face orthogonale (carré, cathète d'un triangle) → demi-tour ;
  hypoténuse d'un triangle → déviation à 90°. Chaque couleur de gemme
  touchée est ajoutée une fois à l'ensemble mélangé (voir table ci-dessous).
  Corps noir → absorption. Diamant → dévie sans altérer la couleur.
- **Tour de jeu** : à son tour, un joueur choisit une action — tirer une
  onde, sonder une case, ou proposer la disposition complète adverse. Une
  proposition fausse coûte une vie (1 ou 2 selon les options de lobby) ;
  à 0 vie, le joueur est éliminé. Une proposition juste gagne
  immédiatement, **sauf** si c'est le premier joueur (dans l'ordre du tour)
  qui trouve en premier : l'adversaire a alors droit à un dernier tour pour
  égaliser.

| Couleurs touchées | Résultat | | Couleurs touchées | Résultat |
|---|---|---|---|---|
| aucune | Transparent | | Rouge+Blanc | Rose |
| Rouge | Rouge | | Bleu+Blanc | Bleu ciel |
| Bleu | Bleu | | Jaune+Blanc | Citron |
| Jaune | Jaune | | Rouge+Bleu+Jaune | Noir |
| Blanc | Blanc | | Rouge+Bleu+Blanc | Violet clair |
| Rouge+Bleu | Violet | | Bleu+Jaune+Blanc | Vert clair |
| Rouge+Jaune | Orange | | Rouge+Jaune+Blanc | Orange clair |
| Bleu+Jaune | Vert | | Les 4 | Gris |

## Configurer les pièces (`data/pieces.json`)

Chaque pièce n'y est décrite que par sa **forme de base** (orientation 0) :
une liste de cases relatives `{x, y, kind}`, où `kind` vaut `"square"` ou
`"tri_nw"` / `"tri_ne"` / `"tri_se"` / `"tri_sw"` (triangle occupant la
moitié de la case adjacente à ce coin). Toutes les orientations valides
(rotations 90°, miroir) sont **générées automatiquement** par le moteur à
partir de cette forme de base et dédupliquées — pas besoin (ni possibilité
sans introduire une divergence) de les lister à la main.

Pour corriger la forme d'une pièce (par exemple si le livret officiel
précise une géométrie différente de celle retenue ici, voir le point
ouvert ci-dessous) : modifie uniquement le tableau `cells` de la pièce
concernée dans `data/pieces.json`, puis relance
`cargo test -p orapa-core` — un test vérifie que chaque orientation
générée a bien la même aire que la forme de base, ce qui détecte la
plupart des erreurs de saisie.

Le fichier définit aussi la dimension de la grille (`grid`) et si la
contrainte de symétrie des blanches est active (`rules.white_symmetry_forbidden`).

## Cas limites du faisceau

Modèle "centre-de-case" : le faisceau se propage de centre de case en
centre de case, entre/sort par le milieu des côtés — il ne passe donc
jamais exactement par un sommet de la grille.

- **Coin exact / faisceau qui longe une arête** : ne peuvent pas se
  produire avec ce modèle (garanti par construction).
- **Corps noir** : absorbe le faisceau (`Absorbed`, ni point de sortie ni
  couleur).
- **Boucle fermée** (rebond en cycle entre plusieurs pièces) : détectée par
  répétition d'un état `(case, direction)` → `Lost` ("onde perdue dans la
  mine"), avec un filet de sécurité supplémentaire (nombre de pas maximum)
  en cas d'évolution future du modèle. Voir
  `crates/orapa-core/src/beam.rs::tests::closed_loop_between_pieces_is_detected_as_lost`
  pour un exemple construit à la main.

## Tests

```sh
cargo test --workspace     # moteur + serveur (unitaires + intégration WS) + wasm (cible hôte)
cargo clippy --workspace --all-targets
cd web && npx tsc -b && npx oxlint src
```

Le moteur (`orapa-core`) est couvert nominal + tous les cas limites listés
plus haut ; le serveur inclut des tests d'intégration WebSocket bout-en-bout
(partie complète à 2 joueurs, reconnexion, mode solo) via un vrai client
`tokio-tungstenite` contre le routeur axum réel.

## Limites connues

- **Formes des 5 pièces de base** : le livret officiel détaillant leur
  géométrie exacte n'était pas accessible au moment du développement ; les
  formes retenues (voir tableau du plan / `data/pieces.json`) sont une
  reconstruction à partir des indications disponibles, à corriger
  facilement via `data/pieces.json` si besoin (voir plus haut).
- **Persistance** : les parties vivent en mémoire dans le processus
  serveur. La reconnexion après rechargement de page ou coupure réseau
  fonctionne (jeton stocké côté client) ; un redémarrage du serveur perd
  toutes les parties en cours.
- **Mode aide** : la vérification de cohérence des hypothèses tourne
  localement dans le navigateur (WebAssembly), à partir de l'historique
  envoyé par le serveur — aucune disposition secrète adverse n'y transite.
