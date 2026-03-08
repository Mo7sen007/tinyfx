import { createHttp } from "@tinyfx/runtime";
import type { UserDto } from "@dto/user.dto";
import { User } from "@models/user.model";

const http = createHttp({ baseUrl: "/api" });

export class UserService {
  async getUser(id: number): Promise<User> {
    const dto = await http.get<UserDto>(`/users/${id}`);
    return User.from(dto);
  }
}
