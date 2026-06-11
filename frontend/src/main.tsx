import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import { registerAppServiceWorker } from './pwa-registration'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)

if ("serviceWorker" in navigator) {
  window.addEventListener("load", () => {
    void registerAppServiceWorker(
      navigator.serviceWorker.register.bind(navigator.serviceWorker),
      new URL("/service-worker.js", window.location.origin),
    );
  });
}
