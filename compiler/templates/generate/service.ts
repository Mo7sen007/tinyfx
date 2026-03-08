// {{Name}} service

import { createHttp } from "@tinyfx/runtime";

const http = createHttp({ baseUrl: "/api" });

export function create{{Name}}Service() {
  return {
    // add methods here
    // example:
    // async getItem(id: number) {
    //   return http.get(`/items/${id}`);
    // }
  };
}

export type {{Name}}Service = ReturnType<typeof create{{Name}}Service>;
