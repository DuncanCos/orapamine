import { createRoot } from "react-dom/client";
import App from "./App.tsx";

// Pas de StrictMode : le montage double-invoque les effets en dev, ce qui
// ouvrirait deux connexions WebSocket concurrentes au chargement.
createRoot(document.getElementById("root")!).render(<App />);
