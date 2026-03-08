// Lightweight page context passed to components by the runtime.
// Not a DI container — services are created explicitly via factory functions.

export interface TinyFxContext {
  params: Record<string, string>;
  navigate: (path: string) => void;
}