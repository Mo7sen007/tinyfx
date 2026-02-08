// Application-level reactive state.
// Import signals from the runtime and define shared state here.

import { signal } from "@tinyfx/runtime";

// Example: a global counter signal
export const count = signal(0);
