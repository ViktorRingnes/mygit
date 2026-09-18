import { createRouter as createTanStackRouter } from "@tanstack/react-router";
import { setupRouterSsrQueryIntegration } from "@tanstack/react-router-ssr-query";

import type { BranchState } from "#/stores/branch";
import { createBranchStore } from "#/stores/branch";

import { getContext } from "./integrations/tanstack-query/root-provider";
import { routeTree } from "./routeTree.gen";

export function getRouter() {
  const { queryClient } = getContext();
  const branchStore = createBranchStore();

  const router = createTanStackRouter({
    routeTree,
    context: { queryClient, branchStore },
    scrollRestoration: true,
    defaultPreload: "intent",
    defaultPreloadStaleTime: 0,
    defaultNotFoundComponent: () => <p>not found</p>,
    dehydrate: (): { branch: BranchState } => ({ branch: branchStore.state }),
    hydrate: ({ branch }: { branch: BranchState }) => {
      branchStore.actions.restore(branch);
    },
  });

  setupRouterSsrQueryIntegration({ router, queryClient });

  return router;
}

declare module "@tanstack/react-router" {
  interface Register {
    router: ReturnType<typeof getRouter>;
  }
}
