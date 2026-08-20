//! Petite démo CLI du moteur : affiche un plateau en ASCII et le trajet
//! d'une onde tirée depuis un point du bord, avec le résultat annoncé.
//!
//! Usage :
//!   cargo run -p orapa-cli -- --seed 1 --beam 7
//!   cargo run -p orapa-cli -- --seed 1 --beam A --diamond --black

use clap::Parser;
use orapa_core::{
    entry_points, fire_beam, random_valid_placement, BeamOutcome, Board, GameOptions,
    PieceCatalog, PointIndex,
};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::collections::HashSet;

#[derive(Parser, Debug)]
#[command(about = "Démo ASCII du moteur Orapa Mine : trace une onde sur un plateau tiré au hasard")]
struct Args {
    /// Graine aléatoire pour le tirage du plateau.
    #[arg(long, default_value_t = 1)]
    seed: u64,

    /// Point de tir (ex: "A", "7", "R18"...). Si omis, liste tous les points.
    #[arg(long)]
    beam: Option<String>,

    /// Inclure l'extension Diamant.
    #[arg(long, default_value_t = false)]
    diamond: bool,

    /// Inclure l'extension Corps noir.
    #[arg(long, default_value_t = false)]
    black: bool,

    /// Trace toutes les 36 ondes au lieu d'une seule.
    #[arg(long, default_value_t = false)]
    all: bool,
}

fn main() {
    let args = Args::parse();
    let catalog = PieceCatalog::default_catalog();
    let mut rng = SmallRng::seed_from_u64(args.seed);
    let options = GameOptions {
        diamond: args.diamond,
        black: args.black,
    };
    let placements = random_valid_placement(&mut rng, &catalog, options);
    let board = Board::build(&catalog, catalog.grid_width, catalog.grid_height, &placements)
        .expect("le placement tiré par random_valid_placement doit être constructible");
    let points = PointIndex::build(catalog.grid_width, catalog.grid_height);

    println!("Plateau tiré (graine {}) :", args.seed);
    println!("{}", render_board(&board, &HashSet::new()));
    println!();

    if args.all {
        for p in entry_points(catalog.grid_width, catalog.grid_height) {
            let outcome = fire_beam(&board, &points, &p.id).unwrap();
            println!("{:>3} -> {}", p.id, describe(&outcome));
        }
        return;
    }

    let beam_id = match &args.beam {
        Some(b) => b.clone(),
        None => {
            println!("Points de tir disponibles :");
            for p in entry_points(catalog.grid_width, catalog.grid_height) {
                print!("{:>3} ", p.id);
            }
            println!();
            println!("(relancer avec --beam <id>, ou --all pour toutes les tracer)");
            return;
        }
    };

    match fire_beam(&board, &points, &beam_id) {
        Some(outcome) => {
            println!("Onde tirée depuis {beam_id} : {}", describe(&outcome));
        }
        None => {
            eprintln!("Point de tir inconnu : {beam_id}");
            std::process::exit(1);
        }
    }
}

fn describe(outcome: &BeamOutcome) -> String {
    match outcome {
        BeamOutcome::Exit { point, color } => {
            format!("ressort en {point}, couleur {}", color.label_fr())
        }
        BeamOutcome::Absorbed => "signal absorbé (corps noir)".to_string(),
        BeamOutcome::Lost => "onde perdue dans la mine (boucle)".to_string(),
    }
}

fn render_board(board: &Board, _highlight: &HashSet<(i32, i32)>) -> String {
    let mut s = String::new();
    for y in 0..board.height {
        for x in 0..board.width {
            let ch = match board.get(x, y) {
                None => '·',
                Some(cell) => match cell.kind {
                    orapa_core::geometry::CellKind::Square => '■',
                    orapa_core::geometry::CellKind::Triangle(c) => match c {
                        orapa_core::geometry::Corner::Nw => '◤',
                        orapa_core::geometry::Corner::Ne => '◥',
                        orapa_core::geometry::Corner::Se => '◢',
                        orapa_core::geometry::Corner::Sw => '◣',
                    },
                },
            };
            s.push(ch);
            s.push(' ');
        }
        s.push('\n');
    }
    s
}
