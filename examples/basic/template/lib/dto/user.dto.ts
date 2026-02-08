// DTO: raw shape coming from the API.
// This matches the backend JSON contract exactly.

export interface UserDto {
  id: number;
  name: string;
  created_at: string;
}
