export function mount(el: HTMLElement) {
  const nameSpan = el.querySelector<HTMLSpanElement>("#name");
  const name = el.getAttribute("data-name") || "World";
  if (nameSpan) {
    nameSpan.textContent = name;
  }
}
