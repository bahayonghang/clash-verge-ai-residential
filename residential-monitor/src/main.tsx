import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./app";
import { TooltipProvider } from "./components/ui/tooltip";
import "./styles/globals.css";

const root = document.querySelector("#app");
if (root instanceof HTMLElement) {
  createRoot(root).render(
    <StrictMode>
      <TooltipProvider>
        <App />
      </TooltipProvider>
    </StrictMode>
  );
}
