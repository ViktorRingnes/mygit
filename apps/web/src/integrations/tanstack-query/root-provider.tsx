import { QueryClient } from "@tanstack/react-query";

export function getContext() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: 1000 * 60,
        retry: false,
      },
    },
  });

  return {
    queryClient,
  };
}
export default function TanstackQueryProvider() {}
