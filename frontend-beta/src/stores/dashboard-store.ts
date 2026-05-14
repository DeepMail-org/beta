import { create } from "zustand";
import { subscribeWithSelector } from "zustand/middleware";
import type {
  WidgetInstance,
  WidgetLayoutItem,
  DashboardLayoutState,
  WidgetDefinition,
} from "@/lib/dashboard/types";
import { STORAGE_KEY } from "@/lib/dashboard/types";

// ── Persisted state shape ──────────────────────────────────────────────────

interface PersistedDashboardState {
  activeWidgets: WidgetInstance[];
  layouts: DashboardLayoutState;
}

// ── Store interface ────────────────────────────────────────────────────────

interface DashboardStore {
  activeWidgets: WidgetInstance[];
  layouts: DashboardLayoutState;
  isMarketplaceOpen: boolean;
  isCommandPaletteOpen: boolean;

  addWidget: (definition: WidgetDefinition, position?: { x: number; y: number }) => void;
  removeWidget: (widgetId: string) => void;
  updateLayout: (breakpoint: keyof DashboardLayoutState, layout: WidgetLayoutItem[]) => void;
  updateAllLayouts: (layouts: DashboardLayoutState) => void;
  setMarketplaceOpen: (open: boolean) => void;
  setCommandPaletteOpen: (open: boolean) => void;
  hydrate: () => void;
}

// ── localStorage helpers ───────────────────────────────────────────────────

function loadFromStorage(): PersistedDashboardState | null {
  if (typeof window === "undefined") return null;
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (
      parsed &&
      Array.isArray(parsed.activeWidgets) &&
      parsed.layouts &&
      typeof parsed.layouts === "object"
    ) {
      return parsed as PersistedDashboardState;
    }
  } catch {
    // Corrupted data — start fresh
  }
  return null;
}

function saveToStorage(state: PersistedDashboardState): void {
  if (typeof window === "undefined") return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch {
    // Storage full or unavailable — non-critical
  }
}

// ── Debounced backend sync (stub) ──────────────────────────────────────────

let syncTimer: ReturnType<typeof setTimeout> | null = null;

function scheduleBackendSync(state: PersistedDashboardState): void {
  if (syncTimer) clearTimeout(syncTimer);
  syncTimer = setTimeout(() => {
    // Future: POST to /api/v1/tenant/dashboard-layout
    // apiFetch("/tenant/dashboard-layout", { method: "PUT", body: JSON.stringify(state) })
    void state;
  }, 2000);
}

// ── Persist middleware (write-through to localStorage + debounced backend) ─

function persist(state: DashboardStore): void {
  const persisted: PersistedDashboardState = {
    activeWidgets: state.activeWidgets,
    layouts: state.layouts,
  };
  saveToStorage(persisted);
  scheduleBackendSync(persisted);
}

// ── Default layouts ────────────────────────────────────────────────────────

const DEFAULT_LAYOUTS: DashboardLayoutState = {
  lg: [],
  md: [],
  sm: [],
  xs: [],
};

// ── Store creation ─────────────────────────────────────────────────────────

export const useDashboardStore = create<DashboardStore>()(
  subscribeWithSelector((set) => ({
    activeWidgets: [],
    layouts: DEFAULT_LAYOUTS,
    isMarketplaceOpen: false,
    isCommandPaletteOpen: false,

    addWidget: (definition, position) => {
      const instanceId = `${definition.id}_${Date.now().toString(36)}`;
      const newWidget: WidgetInstance = {
        id: instanceId,
        definitionId: definition.id,
        layout: {
          i: instanceId,
          x: position?.x ?? 0,
          y: position?.y ?? Infinity,
          w: definition.defaultSize.w,
          h: definition.defaultSize.h,
          minW: definition.minSize.w,
          minH: definition.minSize.h,
          maxW: definition.maxSize.w,
          maxH: definition.maxSize.h,
        },
      };

      set((state) => {
        const activeWidgets = [...state.activeWidgets, newWidget];
        const layouts: DashboardLayoutState = {
          lg: [...state.layouts.lg, newWidget.layout],
          md: [...state.layouts.md, { ...newWidget.layout, w: Math.min(newWidget.layout.w, 8) }],
          sm: [...state.layouts.sm, { ...newWidget.layout, w: Math.min(newWidget.layout.w, 4) }],
          xs: [...state.layouts.xs, { ...newWidget.layout, w: 2 }],
        };
        const next = { activeWidgets, layouts };
        persist({ ...state, ...next });
        return next;
      });
    },

    removeWidget: (widgetId) => {
      set((state) => {
        const activeWidgets = state.activeWidgets.filter((w) => w.id !== widgetId);
        const layouts: DashboardLayoutState = {
          lg: state.layouts.lg.filter((l) => l.i !== widgetId),
          md: state.layouts.md.filter((l) => l.i !== widgetId),
          sm: state.layouts.sm.filter((l) => l.i !== widgetId),
          xs: state.layouts.xs.filter((l) => l.i !== widgetId),
        };
        const next = { activeWidgets, layouts };
        persist({ ...state, ...next });
        return next;
      });
    },

    updateLayout: (breakpoint, layout) => {
      set((state) => {
        const layouts = { ...state.layouts, [breakpoint]: layout };
        const next = { layouts };
        persist({ ...state, ...next });
        return next;
      });
    },

    updateAllLayouts: (layouts) => {
      set((state) => {
        persist({ ...state, layouts });
        return { layouts };
      });
    },

    setMarketplaceOpen: (open) => set({ isMarketplaceOpen: open }),
    setCommandPaletteOpen: (open) => set({ isCommandPaletteOpen: open }),

    hydrate: () => {
      const saved = loadFromStorage();
      if (saved) {
        set({
          activeWidgets: saved.activeWidgets,
          layouts: saved.layouts,
        });
      }
    },
  }))
);
