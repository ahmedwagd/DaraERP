import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./i18n";
import { invoke } from "@tauri-apps/api/core";


// Debug helper for console testing
(window as any).api = { invoke };


ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
