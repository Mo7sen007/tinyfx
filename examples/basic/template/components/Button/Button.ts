// Optional component behavior.
// This runs at runtime for each <data-component="Button"> element.

export function init(el: HTMLElement) {
  el.addEventListener("click", () => {
    console.log("Button clicked:", el.textContent?.trim());
  });
}
