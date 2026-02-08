// tinyfx entry point
// This file runs after the page loads and initializes
// any compiled components that have behavior attached.

// In this monorepo, we import the runtime from the sibling ./runtime folder.
// If you publish tinyfx as a package later, you can switch this to e.g. "tinyfx/runtime".
import { initComponents, signal, effect } from "{{RUNTIME_IMPORT}}";

// Initialize all <data-component> elements on the page
document.addEventListener("DOMContentLoaded", () => {
  initComponents();
});

// Example: reactive counter
const count = signal(0);
const incBtn = document.getElementById("inc");

if (incBtn) {
  incBtn.addEventListener("click", () => {
    count.set(count() + 1);
  });
}

effect(() => {
  const el = document.getElementById("output");
  if (el) {
    el.textContent = `Count: ${count()}`;
  }
});
