"use client";

import { useState, useEffect, useCallback, useRef } from "react";
import { useWidgetContext } from "@/lib/dashboard/widget-context";
import { wsManager } from "@/lib/dashboard/ws-manager";
import type { WidgetDefinition } from "@/lib/dashboard/types";

interface WidgetDataResult<T = unknown> {
  data: T;
  isLoading: boolean;
  error: Error | null;
  refetch: () => void;
}

export function useWidgetData<T = unknown>(definition: WidgetDefinition<T>): WidgetDataResult<T> {
  const { dataSource, useData } = definition;
  const { apiBase, wsBase, token } = useWidgetContext();
  const [data, setData] = useState<T>(null as T);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const mountedRef = useRef(true);

  useEffect(() => {
    mountedRef.current = true;
    return () => { mountedRef.current = false; };
  }, []);

  const fetchData = useCallback(async () => {
    if (!dataSource.endpoint) return;
    setIsLoading(true);
    setError(null);

    try {
      const headers: Record<string, string> = { "Content-Type": "application/json" };
      if (token) headers["Authorization"] = `Bearer ${token}`;

      const res = await fetch(`${apiBase}${dataSource.endpoint}`, { headers });
      if (!res.ok) {
        throw new Error(`API ${res.status}: ${res.statusText}`);
      }
      const json = await res.json();
      if (mountedRef.current) {
        setData(json as T);
        setIsLoading(false);
      }
    } catch (err) {
      if (mountedRef.current) {
        setError(err instanceof Error ? err : new Error(String(err)));
        setIsLoading(false);
      }
    }
  }, [apiBase, token, dataSource.endpoint]);

  // Custom hook delegation
  if (dataSource.type === "custom" && useData) {
    // eslint-disable-next-line react-hooks/rules-of-hooks
    return useData() as WidgetDataResult<T>;
  }

  // Polling data source
  useEffect(() => {
    if (dataSource.type !== "polling") return;

    fetchData();
    const interval = setInterval(fetchData, dataSource.interval ?? 30_000);
    return () => clearInterval(interval);
  }, [dataSource.type, dataSource.interval, fetchData]);

  // GraphQL / one-shot fetch
  useEffect(() => {
    if (dataSource.type !== "graphql") return;
    fetchData();
  }, [dataSource.type, fetchData]);

  // WebSocket data source
  useEffect(() => {
    if (dataSource.type !== "websocket") return;
    if (!dataSource.channel) return;

    wsManager.connect(wsBase);

    const unsubscribe = wsManager.subscribe(dataSource.channel, (msg) => {
      if (mountedRef.current) {
        setData(msg as T);
        setIsLoading(false);
        setError(null);
      }
    });

    return unsubscribe;
  }, [dataSource.type, dataSource.channel, wsBase]);

  return { data, isLoading, error, refetch: fetchData };
}
