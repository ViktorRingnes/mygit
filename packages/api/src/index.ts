import { queryOptions } from "@tanstack/react-query";
import type { components, paths } from "contract";
import createClient, { type ClientOptions, type MaybeOptionalInit } from "openapi-fetch";
import type { HttpMethod, PathsWithMethod, RequiredKeysOf } from "openapi-typescript-helpers";

export type { components, paths } from "contract";

export type Schema<Name extends keyof components["schemas"]> = components["schemas"][Name];

type Path<M extends HttpMethod> = PathsWithMethod<paths, M>;

type Init<M extends HttpMethod, P extends Path<M>> = MaybeOptionalInit<paths[P], M>;

type Args<I> =
  RequiredKeysOf<I> extends never
    ? [(I & Record<string, unknown>)?]
    : [I & Record<string, unknown>];

export class ApiError extends Error {
  override readonly name = "ApiError";
  readonly status: number;

  constructor(response: Response, body: unknown) {
    super((body as Schema<"ErrorResponse"> | undefined)?.message ?? `HTTP ${response.status}`);

    this.status = response.status;
  }
}

export function createApi(options: ClientOptions) {
  const client = createClient<paths>(options);

  const get = <P extends Path<"get">, I extends Init<"get", P>>(path: P, ...args: Args<I>) =>
    unwrap(client.GET<P, I>(path, ...args));

  return {
    client,
    get,
    post: <P extends Path<"post">, I extends Init<"post", P>>(path: P, ...args: Args<I>) =>
      unwrap(client.POST<P, I>(path, ...args)),
    put: <P extends Path<"put">, I extends Init<"put", P>>(path: P, ...args: Args<I>) =>
      unwrap(client.PUT<P, I>(path, ...args)),
    patch: <P extends Path<"patch">, I extends Init<"patch", P>>(path: P, ...args: Args<I>) =>
      unwrap(client.PATCH<P, I>(path, ...args)),
    delete: <P extends Path<"delete">, I extends Init<"delete", P>>(path: P, ...args: Args<I>) =>
      unwrap(client.DELETE<P, I>(path, ...args)),
    query: <P extends Path<"get">, I extends Init<"get", P>>(path: P, ...args: Args<I>) =>
      queryOptions({
        queryKey: [path, args[0]?.params ?? null],
        refetchOnWindowFocus: false,
        queryFn: () => get<P, I>(path, ...args),
      }),
  };
}

export type Api = ReturnType<typeof createApi>;

async function unwrap<T>(pending: Promise<{ data?: T; error?: unknown; response: Response }>) {
  const { data, error, response } = await pending;

  if (!response.ok) throw new ApiError(response, error);

  return data as T;
}
