// User service

import { createHttp } from "@tinyfx/runtime";

const http = createHttp({ baseUrl: "/api" });

export function createUserService() {
  return {
    // add methods here
    // example:
    // async getItem(id: number) {
    //   return http.get(`/items/${id}`);
    // }
  };
}

export type UserService = ReturnType<typeof createUserService>;
