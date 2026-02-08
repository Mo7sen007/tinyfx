import { signal, effect } from "../../runtime";

const count = signal(0);

document.getElementById("btn")!.onclick = () => {
  count.set(count() + 1);
};

effect(() => {
  document.getElementById("out")!.textContent = String(count());
});
