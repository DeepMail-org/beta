"use client";

import { createContext, useContext, useMemo, type ReactNode } from "react";
import { getStoredToken } from "@/lib/api";

interface WidgetContextValue {
  token: string | null;
  apiBase: string;
  wsBase: string;
}

const WidgetContext = createContext<WidgetContextValue>({
  token: null,
  apiBase: "",
  wsBase: "",
});

function resolveApiBase(): string {
  if (process.env.NEXT_PUBLIC_API_URL) return process.env.NEXT_PUBLIC_API_URL;
  if (typeof window !== "undefined") {
    const protocol = window.location.protocol === "https:" ? "https" : "http";
    return `${protocol}://${window.location.hostname}:3001/api/v1`;
  }
  return "http://localhost:3001/api/v1";
}

function resolveWsBase(): string {
  if (process.env.NEXT_PUBLIC_WS_URL) return process.env.NEXT_PUBLIC_WS_URL;
  if (typeof window !== "undefined") {
    const protocol = window.location.protocol === "https:" ? "wss" : "ws";
    return `${protocol}://${window.location.hostname}:3001/api/v1`;
  }
  return "ws://localhost:3001/api/v1";
}

export function WidgetProvider({ children }: { children: ReactNode }) {
  const value = useMemo<WidgetContextValue>(
    () => ({
      token: getStoredToken(),
      apiBase: resolveApiBase(),
      wsBase: resolveWsBase(),
    }),
    []
  );

  return <WidgetContext.Provider value={value}>{children}</WidgetContext.Provider>;
}

export function useWidgetContext(): WidgetContextValue {
  return useContext(WidgetContext);
}
