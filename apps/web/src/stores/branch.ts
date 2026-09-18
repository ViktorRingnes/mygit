import { createStore } from "@tanstack/store";

import type { Branches } from "#/lib/queries";

export interface BranchState {
  current: string | null;
}

export type BranchStore = ReturnType<typeof createBranchStore>;

export function createBranchStore() {
  const initial: BranchState = { current: null };

  return createStore(initial, ({ setState }) => ({
    sync: ({ names, default_branch }: Branches) =>
      setState(({ current }) => ({ current: current ?? default_branch ?? names.at(0) ?? null })),
    select: (current: string) => setState(() => ({ current })),
    restore: (state: BranchState) => setState(() => state),
  }));
}
