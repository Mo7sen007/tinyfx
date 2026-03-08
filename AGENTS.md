## Import aliases

All lib imports use path aliases — never use relative paths to lib files.

| Alias        | Folder                  | Contains                      |
|--------------|-------------------------|-------------------------------|
| @models/*    | src/lib/models/         | Frontend model classes        |
| @dto/*       | src/lib/dto/            | Raw API response types        |
| @state/*     | src/lib/state/          | Signals and global state      |
| @services/*  | src/lib/services/       | HTTP + business logic classes |

Examples:
import { User } from "@models/user.model";
import type { UserDto } from "@dto/user.dto";
import { UserService } from "@services/user.service";

There are no index.ts barrel files. Always import the file directly.
