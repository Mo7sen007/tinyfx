// tinyfx entry point
// Runs after the page loads and mounts all class-based components.
import { signal, effect } from "@tinyfx/runtime";
import { Container } from "@tinyfx/runtime/di";
import { setupComponents } from "./tinyfx.gen";

// 1. Create a DI container and register services
const container = new Container();
// container.provide(MyService, new MyService());

// 2. Mount all <data-component> elements on the page
document.addEventListener("DOMContentLoaded", () => {
  const instances = setupComponents(container);
  // `instances` holds every mounted component for optional teardown
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
