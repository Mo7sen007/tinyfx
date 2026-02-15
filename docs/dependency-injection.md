# Dependency Injection

TinyFX features a small, decoupled Dependency Injection (DI) system to manage services and state across your application.

## The Container

The `Container` is where you register your services. Usually, you create one in your `main.ts`.

```typescript
import { Container } from "@tinyfx/runtime/di";
import { UserService } from "./services/user.service";

const container = new Container();

// Register a service instance
container.provide(UserService, new UserService());
```

## Using Tokens

You can use Classes or Symbols as tokens to register services.

```typescript
import { createToken } from "@tinyfx/runtime/di";

// Create a unique token
export const API_KEY = createToken<string>("API_KEY");

container.provide(API_KEY, "secret-123");
```

## Injecting in Components

Every component receives the `TinyFxContext` in its constructor, which contains the DI container.

```typescript
import { UserService } from "../../lib/services/user.service";
import { TinyFxContext } from "@tinyfx/runtime/di";

export class UserProfile {
  private userService: UserService;

  constructor(public el: HTMLElement, public ctx: TinyFxContext) {
    // Retrieve the service from the container
    this.userService = this.ctx.container.inject(UserService);
  }

  async init() {
    const user = await this.userService.getCurrentUser();
    // ...
  }
}
```

## Initializing the App

In your `main.ts`, you pass the container to the generated `setupComponents` function.

```typescript
import { Container } from "@tinyfx/runtime/di";
import { setupComponents } from "./tinyfx.gen";

const container = new Container();
// ... provide services ...

document.addEventListener("DOMContentLoaded", () => {
  // This automatically injects the container into all mounted components
  setupComponents(container);
});
```
