import type { Schema } from "api";

import { api } from "#/lib/api";

export type Branches = Schema<"Branches">;
export type Tree = Schema<"Tree">;
export type Entry = Schema<"Entry">;

export interface TreeParams {
  rev?: string | undefined;
  path?: string | undefined;
}

export const branchesQuery = api.query("/branches");

export const treeQuery = ({ rev, path }: TreeParams) =>
  api.query("/tree", { params: { query: { rev, path } } });
