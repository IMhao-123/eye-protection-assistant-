export type ViewMode = "main" | "widget" | "break";

export function resolveViewMode(search: string): ViewMode {
  const view = new URLSearchParams(search).get("view");
  return view === "widget" || view === "break" ? view : "main";
}

export function applyDocumentSurface(view: ViewMode, target: Document = document) {
  target.documentElement.dataset.view = view;
  const background = view === "widget" ? "transparent" : "";
  target.documentElement.style.backgroundColor = background;
  target.body.style.backgroundColor = background;
  const root = target.getElementById("root");
  if (root) root.style.backgroundColor = background;
}
