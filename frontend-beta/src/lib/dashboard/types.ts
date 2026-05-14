import type { ComponentType, LazyExoticComponent } from "react";

// ── Layout item (matches react-grid-layout LayoutItem) ─────────────────────

export interface WidgetLayoutItem {
  i: string;
  x: number;
  y: number;
  w: number;
  h: number;
  minW?: number;
  minH?: number;
  maxW?: number;
  maxH?: number;
  static?: boolean;
}

// ── Data source configuration ──────────────────────────────────────────────

export type DataSourceType = "websocket" | "polling" | "graphql" | "custom";

export interface WidgetDataSource {
  type: DataSourceType;
  endpoint?: string;
  interval?: number;
  channel?: string;
}

// ── Widget plugin interface ────────────────────────────────────────────────

export interface WidgetSize {
  w: number;
  h: number;
}

export type WidgetCategory =
  | "analytics"
  | "threats"
  | "pipeline"
  | "intelligence"
  | "compliance"
  | "custom";

export interface WidgetProps<T = unknown> {
  widgetId: string;
  data: T;
  isLoading: boolean;
  error: Error | null;
  refetch: () => void;
}

import type { LucideIcon } from "lucide-react";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export interface WidgetDefinition<T = any> {
  id: string;
  name: string;
  description: string;
  icon: LucideIcon;
  category: WidgetCategory;
  defaultSize: WidgetSize;
  minSize: WidgetSize;
  maxSize: WidgetSize;
  dataSource: WidgetDataSource;
  component: LazyExoticComponent<ComponentType<WidgetProps<T>>>;
  useData?: () => { data: T; isLoading: boolean; error: Error | null; refetch: () => void };
}

// ── Widget instance (an active widget placed on the dashboard) ─────────────

export interface WidgetInstance {
  id: string;
  definitionId: string;
  layout: WidgetLayoutItem;
}

// ── Dashboard layout (per-breakpoint) ──────────────────────────────────────

export interface DashboardLayoutState {
  lg: WidgetLayoutItem[];
  md: WidgetLayoutItem[];
  sm: WidgetLayoutItem[];
  xs: WidgetLayoutItem[];
}

// ── Breakpoint config ──────────────────────────────────────────────────────

export const BREAKPOINTS = { lg: 1200, md: 996, sm: 768, xs: 480 } as const;
export const COLS = { lg: 12, md: 8, sm: 4, xs: 2 } as const;
export const ROW_HEIGHT = 180;
export const GRID_MARGIN: [number, number] = [16, 16];

// ── Persistence key ────────────────────────────────────────────────────────

export const STORAGE_KEY = "deepmail_dashboard_v2" as const;
