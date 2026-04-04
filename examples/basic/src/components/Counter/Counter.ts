import { signal, effect } from "@tinyfx/runtime";

export function mount(el: HTMLElement) {
  const count = signal(0);

  const decrementBtn = el.querySelector<HTMLButtonElement>("#decrement");
  const incrementBtn = el.querySelector<HTMLButtonElement>("#increment");
  const countSpan = el.querySelector<HTMLSpanElement>("#count");

  if (decrementBtn) {
    decrementBtn.addEventListener("click", () => count.set(count() - 1));
  }

  if (incrementBtn) {
    incrementBtn.addEventListener("click", () => count.set(count() + 1));
  }

  if (countSpan) {
    effect(() => {
      countSpan.textContent = String(count());
    });
  }
}
