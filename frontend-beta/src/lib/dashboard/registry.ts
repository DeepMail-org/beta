import type { WidgetDefinition } from "./types";

class WidgetRegistry {
  private definitions = new Map<string, WidgetDefinition>();

  register(definition: WidgetDefinition): void {
    this.definitions.set(definition.id, definition);
  }

  unregister(id: string): void {
    this.definitions.delete(id);
  }

  get(id: string): WidgetDefinition | undefined {
    return this.definitions.get(id);
  }

  getAll(): WidgetDefinition[] {
    return Array.from(this.definitions.values());
  }

  getByCategory(category: string): WidgetDefinition[] {
    return this.getAll().filter((d) => d.category === category);
  }

  has(id: string): boolean {
    return this.definitions.has(id);
  }
}

export const widgetRegistry = new WidgetRegistry();
