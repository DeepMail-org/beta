"use client";

import { Component, type ErrorInfo, type ReactNode } from "react";

interface Props {
  widgetId: string;
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class WidgetErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false, error: null };

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error(`[Widget ${this.props.widgetId}]`, error, info.componentStack);
  }

  private handleRetry = () => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (this.state.hasError) {
      return (
        <div className="h-full w-full rounded-xl border border-danger/20 bg-danger/5 flex flex-col items-center justify-center gap-3 p-6">
          <div className="w-10 h-10 rounded-full bg-danger/10 flex items-center justify-center">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="text-danger">
              <circle cx="12" cy="12" r="10" />
              <line x1="12" y1="8" x2="12" y2="12" />
              <line x1="12" y1="16" x2="12.01" y2="16" />
            </svg>
          </div>
          <p className="text-sm text-muted text-center">Something went wrong</p>
          <p className="text-xs text-muted/60 text-center max-w-[200px] truncate">
            {this.state.error?.message}
          </p>
          <button
            type="button"
            onClick={this.handleRetry}
            className="mt-1 text-xs px-3 py-1.5 rounded-lg bg-surface-2 border border-border text-foreground hover:bg-surface-2/80 transition-colors"
          >
            Retry
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}
