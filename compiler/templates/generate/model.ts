import type { {{Name}}Dto } from "@dto/{{name}}.dto";

export class {{Name}} {
  constructor(
    public id: number,
    public name: string,
    public createdAt: Date,
  ) {}

  static from(dto: {{Name}}Dto): {{Name}} {
    return new {{Name}}(dto.id, dto.name, new Date(dto.created_at));
  }
}
