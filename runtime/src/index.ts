// @tinyfx/runtime — barrel export
export { signal, effect, Signal } from "./signals";
export { bindText, bindAttr, bindClass } from "./dom";
export { createHttp } from "./http/http";
export type { HttpConfig } from "./http/data";
export { Container, createToken } from "./di";
export type { Token, TinyFxContext } from "./di";

// Router — hard file-based router
export { initRouter } from "./router/index";
export { getParam } from "./router/params";
export { onMount, onDestroy } from "./router/lifecycle";
export { navigate, goBack } from "./router/navigate";
export type { RouteMap, RouteMeta } from "./router/types";
