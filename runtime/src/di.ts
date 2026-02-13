// @tinyfx/runtime — Dependency Injection container

/** A token used to register and retrieve a service from the container. */
export type Token<T> = (new (...args: any[]) => T) | symbol;

/** Create a unique symbol token with a debug description. */
export function createToken<T>(description: string): Token<T> {
  return Symbol(description);
}

/** Context passed to every component's mount() factory function. */
export interface TinyFxContext {
  container: Container;
}

/**
 * Lightweight DI container.
 *
 * Register services at bootstrap time with `provide()`,
 * then retrieve them inside components via `inject()`.
 */
export class Container {
  private readonly services = new Map<Token<any>, any>();

  /** Register a service instance under the given token. */
  provide<T>(token: Token<T>, instance: T): void {
    this.services.set(token, instance);
  }

  /** Retrieve a service instance. Throws if not registered. */
  inject<T>(token: Token<T>): T {
    if (!this.services.has(token)) {
      throw new Error(
        `TinyFX: No provider for ${typeof token === "symbol" ? token.toString() : (token as Function).name ?? token}`
      );
    }
    return this.services.get(token) as T;
  }

  /** Check whether a token has been provided. */
  has<T>(token: Token<T>): boolean {
    return this.services.has(token);
  }
}