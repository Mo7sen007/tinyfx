export function initComponents(root: Document | HTMLElement = document) {
  root.querySelectorAll("[data-component]").forEach((el: Element) => {
    // compiler will fill this later
  });
}
