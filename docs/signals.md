# Signals & Reactivity

TinyFX provides a minimal, explicit reactivity system based on **Signals**.

## What are Signals?

A Signal is a wrapper around a value that tracks when it changes and who is interested in that value. It's similar to `Ref` in Vue or `Signal` in SolidJS/Angular.

## Basic Usage

### creating a Signal
Use the `signal()` function to create a new reactive value.

```typescript
import { signal } from "@tinyfx/runtime";

const count = signal(0);
```

### Reading a Signal
Call the signal as a function to get its current value.

```typescript
console.log(count()); // 0
```

### Updating a Signal
Use the `.set()` method to update the value.

```typescript
count.set(count() + 1);
```

## Effects

An `effect()` is a function that runs automatically whenever any signals it "depends" on change.

```typescript
import { signal, effect } from "@tinyfx/runtime";

const count = signal(0);

effect(() => {
  console.log("The count is now:", count());
});

count.set(1); // Logs: "The count is now: 1"
count.set(2); // Logs: "The count is now: 2"
```

### How it Works
When an `effect` runs, TinyFX tracks which signals are called. It then adds the effect as a subscriber to those signals. When a signal is updated via `.set()`, it notifies all its subscribers to re-run.

## Guidelines

- **Always initialize signals immediately**: Do this in your class properties or constructor.
- **Avoid side effects inside effects**: Effects should primarily be used to sync state with the DOM or other external systems.
- **Minimalistic**: TinyFX signals don't use Proxies, so you must always use the function call `()` to read and `.set()` to write.
