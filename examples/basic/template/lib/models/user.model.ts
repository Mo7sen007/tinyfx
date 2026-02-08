// Model: frontend-safe object built from a DTO.
// Handles type conversions (e.g. string → Date).

import type { UserDto } from "../dto/user.dto";

export class User {
  constructor(
    public id: number,
    public name: string,
    public createdAt: Date,
  ) {}

  static from(dto: UserDto): User {
    return new User(dto.id, dto.name, new Date(dto.created_at));
  }
}
