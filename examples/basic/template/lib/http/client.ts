// HTTP client — thin typed wrapper over fetch.
// Configure your base URL and use api.get/post throughout the app.

import { createHttp } from "@tinyfx/runtime";

export const api = createHttp({
  baseUrl: "/api",
});
