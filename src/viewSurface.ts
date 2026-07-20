export type ViewMode = "main" | "widget" | "break";

export function resolveViewMode(search: string): ViewMode {
  const view = new URLSearchParams(search).get("view");
  return view === "widget" || view === "break" ? view : "main";
}

export function applyDocumentSurface(view: ViewMode, target: Document = document) {
  target.documentElement.dataset.view = view;
}
