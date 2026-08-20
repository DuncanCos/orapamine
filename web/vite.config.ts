import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      // En dev, le client tourne sur le port de Vite (5173) mais le
      // serveur de jeu écoute sur :8080 — on relaie le WebSocket pour que
      // `defaultWsUrl()` (qui cible l'origine courante) fonctionne sans
      // configuration supplémentaire.
      '/ws': {
        target: 'ws://127.0.0.1:8080',
        ws: true,
      },
    },
  },
})
