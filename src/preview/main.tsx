import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { Preview } from "./Preview";
import "./preview.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <Preview />
  </StrictMode>,
);
