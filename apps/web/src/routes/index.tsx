import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";

import { api } from "#/lib/api";

export const Route = createFileRoute("/")({ component: Home });

function Home() {
  const { data, error, isPending } = useQuery(api.query("/branches"));

  return (
    <div className="p-8">
      <h1 className="mb-4 text-lg font-medium">Branches</h1>

      {isPending && <p className="text-muted-foreground">Loading</p>}
      {error && <p className="text-destructive">{error.message}</p>}

      <ul className="space-y-1">
        {data?.map((branch) => (
          <li key={branch} className="font-mono text-sm">
            {branch}
          </li>
        ))}
      </ul>
    </div>
  );
}
