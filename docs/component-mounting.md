# Component Mounting

TinyFX mounts dynamic components using **compile-time discovery** and **direct mount calls**. The compiler knows every component name, where it appears, and generates targeted mount instructions. The runtime executes them — no blind DOM scanning required.

---

## Overview

The mounting system has two phases:

| Phase | What happens | Where |
|---|---|---|
| **Compile-time** | Compiler scans HTML, finds all `[data-component]` attributes, builds a sorted list of dynamic component names | `compiler/src/build/` |
| **Runtime** | Generated code calls mount functions directly for each known component | `tinyfx.gen.ts` |

This is fundamentally different from frameworks that scan the DOM at runtime with `querySelectorAll("[data-component]")`. TinyFX generates one targeted query per known component name.

---

## Compile-Time Discovery

### Step 1: Dependency Graph

The build starts by scanning `src/components/` and `src/pages/` to build a dependency graph. Each component HTML file is read and its references to other components are extracted.

```
src/components/Chat/Chat.html    → references: ChatMessage
src/components/ChatMessage/ChatMessage.html → references: (none)
src/pages/index.html             → references: Chat
```

### Step 2: Component Compilation

Components are compiled in dependency order. For each component the compiler determines:

- **Is it dynamic?** — Does the root element have `tinyfx-dynamic`?
- **Directive sites** — Where are `<each>` and `<if>` blocks?
- **Dynamic component names** — Which dynamic components appear inside `<each>` templates and `<if>` branches?

The result is a `CompiledComponent` struct:

```rust
pub struct CompiledComponent {
    pub name: String,              // "Chat"
    pub path: PathBuf,             // src/components/Chat/Chat.html
    pub html: String,              // Fully expanded template
    pub is_dynamic: bool,          // true
    pub each_sites: Vec<EachSite>, // <each> blocks inside
    pub if_sites: Vec<IfSite>,     // <if> blocks inside
}
```

### Step 3: Stamping `data-component`

For every dynamic component usage, the compiler injects a `data-component` attribute into the rendered HTML:

```html
<!-- Source -->
<button class="theme-toggle" tinyfx-dynamic>
  <i class="fa-solid fa-moon"></i>
</button>

<!-- Compiled output -->
<button class="theme-toggle" tinyfx-dynamic data-component="ThemeToggle">
  <i class="fa-solid fa-moon"></i>
</button>
```

This attribute is the **mount marker**. The generated code uses it as a query selector.

### Step 4: Collecting All Dynamic Names

The glue generator (`glue.rs`) collects every dynamic component name from three sources:

1. **Top-level components** — Every component with `is_dynamic: true` in the registry
2. **`<each>` templates** — `EachSite.dynamic_component_names` (components inside loop templates)
3. **`<if>` branches** — `IfBranch.dynamic_component_names` (components inside conditional branches)

The result is a sorted, deduplicated list: `["Chat", "ThemeToggle", "UserProfile"]`.

---

## Code Generation

### Bootstrap Mount Calls

The generated `tinyfx.gen.ts` emits one `querySelectorAll` per known component in the bootstrap function:

```typescript
const boot = () => {
  const ctx = init({ routes, setupDirectives });
  if (!ctx) return;

  document.querySelectorAll<HTMLElement>("[data-component='Chat']").forEach((el) => {
    __mount_Chat(el, ctx);
  });
  document.querySelectorAll<HTMLElement>("[data-component='ThemeToggle']").forEach((el) => {
    __mount_ThemeToggle(el, ctx);
  });
};
```

Key properties:
- **`querySelectorAll`**, not `querySelector` — handles multiple instances safely
- **Per-component query** — targeted, not a blind scan of all `[data-component]`
- **Early return on null** — if no route matches, nothing mounts

### Per-Component Wrappers

For each dynamic component, the compiler generates a `__mount_ComponentName` wrapper:

```typescript
function __mount_Chat(el: HTMLElement, ctx: TinyFxContext): any {
  const inst = mountComponent(ChatModule, el, ctx);
  if (!inst) return null;

  // Wire <each> directive sites
  const __nodes_chat_each_0 = new Map<string, NodeRecord_chat_each_0>();
  const __each_chat_each_0 = el.querySelector<HTMLElement>("[data-each='chat_each_0']");
  if (__each_chat_each_0) {
    effect(() => __reconcile_chat_each_0(__each_chat_each_0, inst.messages(), ctx, __nodes_chat_each_0));
  }

  // Wire <if> directive sites
  effect(() => __toggle_chat_if_0(inst));

  return inst;
}
```

The wrapper:
1. Calls the module's `mount()` or `init()` function
2. Registers the instance for lifecycle tracking
3. Wires up directive sites (`<each>`, `<if>`) to instance signals

### `<each>` Template Mounts

When a component appears inside an `<each>` template, the generated reconciliation code mounts it directly when inserting new items:

```typescript
// Inside __reconcile_chat_each_0
container.insertBefore(record.el, currentAtIndex ?? null);
record.el.querySelectorAll<HTMLElement>("[data-component='ChatMessage']").forEach((node) => {
  __mount_ChatMessage(node, ctx);
});
```

The compiler knows `ChatMessage` appears in the template because it parsed `dynamic_component_names` during compilation.

### `<if>` Branch Mounts

When a conditional branch becomes visible, the toggle function mounts components inside it:

```typescript
// Inside __toggle_chat_if_0
const activeBranch = branches[activeIndex] as HTMLElement;
activeBranch.querySelectorAll<HTMLElement>("[data-component='UserProfile']").forEach((node) => {
  __mount_UserProfile(node, ctx);
});
```

---

## Runtime Execution

### Initialization Flow

```
DOMContentLoaded
    ↓
boot()
    ↓
init({ routes, setupDirectives })
    ├── Match URL against routes
    ├── Init lifecycle hooks (onMount/onDestroy)
    ├── Highlight active <a> links
    ├── Set route params
    ├── Run setupDirectives(ctx) — page-level <each>/<if> wiring
    └── Run page init()
    ↓
Returns TinyFxContext | null
    ↓ (if not null)
Direct mount calls
    ├── querySelectorAll("[data-component='Chat']") → __mount_Chat(el, ctx)
    ├── querySelectorAll("[data-component='ThemeToggle']") → __mount_ThemeToggle(el, ctx)
    └── ...
```

### Mount Protection

The `markMounted()` helper uses a `WeakSet<HTMLElement>` to prevent double-mounting:

```typescript
function mountComponent(module: ComponentModule, el: HTMLElement, ctx: TinyFxContext) {
  if (!markMounted(el)) return undefined;  // Already mounted, skip
  // ...
}
```

This is the single guard against re-mounting. There is no `data-tinyfx-mounted` attribute — the `WeakSet` is invisible to the DOM and gets garbage collected with the element.

### Instance Lifecycle

Mounted instances are tracked in a `WeakMap<HTMLElement, Instance>`:

```typescript
const __componentInstances = new WeakMap<HTMLElement, { destroy?: () => void }>();
```

When a component is destroyed (e.g., removed by `<each>` reconciliation or page navigation), its `destroy()` method is called automatically:

```typescript
function __destroyNode(el: HTMLElement): void {
  const inst = __componentInstances.get(el);
  inst?.destroy?.();
  __componentInstances.delete(el);
  // Also destroys nested [data-component] children
}
```

---

## Mount Ordering

Components mount in **alphabetical order by name** (sorted during code generation). This ensures deterministic behavior:

```typescript
// Always mounts Button before Chat before ThemeToggle
document.querySelectorAll<HTMLElement>("[data-component='Button']").forEach(...);
document.querySelectorAll<HTMLElement>("[data-component='Chat']").forEach(...);
document.querySelectorAll<HTMLElement>("[data-component='ThemeToggle']").forEach(...);
```

Within a single component instance, mounting follows this order:

1. Component `mount()` / `init()` runs
2. `<each>` directive sites are wired (effects created)
3. `<if>` directive sites are wired (effects created or initial toggle runs)
4. Instance is returned

---

## Edge Cases

### Multiple Instances of the Same Component

Using `querySelectorAll` ensures all instances mount:

```html
<Chat />
<Chat />
<Chat />
```

Generates:
```typescript
document.querySelectorAll<HTMLElement>("[data-component='Chat']").forEach((el) => {
  __mount_Chat(el, ctx);
});
```

Each call to `__mount_Chat` receives a different `el` and creates a separate instance with its own directive wiring.

### Nested Components

If `Chat` contains `ChatMessage` as a static (expanded) component, it's already in the HTML and mounts as part of the `Chat` query. If `ChatMessage` is dynamic:

```html
<!-- Chat.html -->
<div tinyfx-dynamic>
  <ChatMessage />
</div>
```

The `ChatMessage` element gets its own `data-component="ChatMessage"` attribute. Since it's inside the `Chat` element, it mounts when the `Chat` query runs — but `__mount_Chat` is called first, then `__mount_ChatMessage`. The alphabetical ordering ensures `Chat` mounts before `ChatMessage`.

### Components Inside `<each>` Loops

Components in loop templates are **not** in the DOM at initial mount time. They are mounted during reconciliation when new items are inserted:

```html
<each of="messages()" as="message" key="id">
  <ChatMessage />
</each>
```

The generated `__reconcile_*` function calls `__mount_ChatMessage` for each newly created element.

### Components Inside `<if>` Branches

Components in conditional branches are **hidden** at mount time (`hidden` attribute). They mount when the branch becomes visible:

```html
<if condition="isLoggedIn()">
  <Dashboard />
<else>
  <Login />
</if>
```

The generated `__toggle_*` function mounts components in the active branch when called.

---

## Runtime API

### Primary APIs (used by generated code)

| Export | Purpose |
|---|---|
| `registerComponent(name, factory)` | Register a mount factory by name |
| `markMounted(el)` | Mark element as mounted (prevents double-mount) |
| `mountComponent(module, el, ctx)` | Internal helper — calls `mount()` or `init()` on a module |

### Escape Hatch APIs (for runtime dynamic HTML)

| Export | Purpose |
|---|---|
| `mountComponentByName(name, el, ctx)` | Mount a single component by name on a specific element |
| `mountComponentsIn(root, ctx, names?)` | Mount all components found in a subtree (optionally filtered by name) |

### Deprecated

| Export | Status | Replacement |
|---|---|---|
| `mountComponents(ctx, root?)` | **Deprecated** | Use generated mount calls or `mountComponentsIn()` |

### When to Use Escape Hatches

```typescript
// You fetched HTML from the server and inserted it
const response = await fetch("/fragment");
const html = await response.text();
container.innerHTML = html;

// Mount components inside the inserted HTML
mountComponentsIn(container, ctx, ["Chat", "UserProfile"]);

// Or mount a single known component
const chatEl = container.querySelector<HTMLElement>("[data-component='Chat']");
if (chatEl) mountComponentByName("Chat", chatEl, ctx);
```

---

## What Changed (Migration Notes)

### `init()` return type

`init()` now returns `TinyFxContext | null` instead of `InitResult`. The `instances` field was removed because the generated code handles mounting directly.

### Page-level `<if>` toggle functions

Toggle functions for page-level `<if>` sites now accept `ctx: TinyFxContext` as a parameter. This was required because they need to mount components inside branches and the `ctx` is no longer available from a global `mountComponents` call.

### `<each>` and `<if>` component mounting

The generated code no longer uses a `registry` variable (which was undefined and broken). It now calls `__mount_ComponentName` directly for each known component in templates.

### `mountComponents()` is deprecated

The runtime discovery function `mountComponents()` is kept for backward compatibility but delegates to `mountComponentsIn()`. New code should use the generated mount calls or `mountComponentsIn()` for dynamic HTML.
