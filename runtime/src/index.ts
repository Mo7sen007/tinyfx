// @tinyfx/runtime — barrel export
export { signal, effect } from "./signals";
export { registerInstance, destroyNode } from "./each";
export { bindText, bindAttr, bindClass } from "./dom";
export { createHttp } from "./http";

// New decoupled APIs
export { registerComponent, getComponentFactory, mountComponentByName, mountComponentsIn, mountComponents } from "./registry";
export { registerPage, getPageModule, runPageInit } from "./page-registry";
export { markMounted, isMounted } from "./mount-state";
export { matchPath, splitPath } from "./router/path-matcher";
export type { RouteDef } from "./router/path-matcher";
export { init } from "./init";
export type { InitConfig } from "./init";

// Keep existing router exports
export { getParam, getParams, navigate, goBack } from "./router/index";
export { onMount, onDestroy } from "./router/lifecycle";
export type { TinyFxContext } from "./context";
export type { RouteMap, RouteMeta } from "./router/index";
