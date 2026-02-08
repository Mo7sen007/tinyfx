// tinyfx entry point
// This file runs after the page loads and initializes
// any compiled components that have behavior attached.
import { signal, effect } from "@tinyfx/runtime";
import { initComponents } from "./tinyfx.gen";

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
