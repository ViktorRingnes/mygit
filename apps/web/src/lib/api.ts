import { createApi } from "api";

import { env } from "#/env";

export const api = createApi({ baseUrl: env.VITE_API_URL });
