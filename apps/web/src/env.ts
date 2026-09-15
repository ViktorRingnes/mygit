import { z } from "zod";

export const env = z
  .object({ VITE_API_URL: z.url().default("http://localhost:5000") })
  .parse(import.meta.env);
