import { signal } from "@tinyfx/runtime";

export const currentUser = signal<string | null>(null);
