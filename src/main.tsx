import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { applyDocumentSurface, resolveViewMode } from "./viewSurface";
import "./styles.css";

applyDocumentSurface(resolveViewMode(window.location.search));

const root = document.getElementById("root");
if (!root) throw new Error("Missing application root");

ReactDOM.createRoot(root).render(
  <React.StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </React.StrictMode>,
);
