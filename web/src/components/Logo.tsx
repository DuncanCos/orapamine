// Emblème du jeu : une gemme à quatre facettes construite avec la même
// géométrie que les pièces posées sur le plateau (voir TRIANGLE_POINTS dans
// Board.tsx) — un carré coupé par ses deux diagonales, tourné à 45°, une
// couleur de gemme par facette. Cohérent avec l'identité visuelle du jeu
// plutôt qu'un logo générique.

interface LogoProps {
  size?: number;
}

export function Logo({ size = 40 }: LogoProps) {
  return (
    <div className="logo" style={{ "--logo-size": `${size}px` } as React.CSSProperties}>
      <svg className="logo-mark" viewBox="0 0 100 100" aria-hidden="true">
        <g transform="rotate(45 50 50)">
          <polygon points="20,20 80,20 50,50" fill="#d3392f" stroke="#1a1712" strokeWidth={3} strokeLinejoin="round" />
          <polygon points="80,20 80,80 50,50" fill="#e8b93a" stroke="#1a1712" strokeWidth={3} strokeLinejoin="round" />
          <polygon points="80,80 20,80 50,50" fill="#2f6fd3" stroke="#1a1712" strokeWidth={3} strokeLinejoin="round" />
          <polygon points="20,80 20,20 50,50" fill="#f2ede1" stroke="#1a1712" strokeWidth={3} strokeLinejoin="round" />
        </g>
      </svg>
      <span className="logo-wordmark">Orapa Mine</span>
    </div>
  );
}
