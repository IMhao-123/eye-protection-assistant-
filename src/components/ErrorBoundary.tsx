import { Component, type ErrorInfo, type ReactNode } from "react";

interface State {
  error: Error | null;
}

export class ErrorBoundary extends Component<{ children: ReactNode }, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("Application render failed", error, info);
  }

  render() {
    if (!this.state.error) return this.props.children;
    return (
      <main className="fatal-error">
        <div className="fatal-error__mark" aria-hidden="true" />
        <h1>护眼助手暂时无法显示</h1>
        <p>计时后台仍会尽量保持运行。重新载入界面通常可以恢复。</p>
        <button className="button button--primary" onClick={() => window.location.reload()}>
          重新载入
        </button>
      </main>
    );
  }
}
