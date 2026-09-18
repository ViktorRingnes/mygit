import { useSuspenseQuery } from "@tanstack/react-query";
import type { ErrorComponentProps } from "@tanstack/react-router";
import { Link, createFileRoute, notFound } from "@tanstack/react-router";
import { useStore } from "@tanstack/react-store";
import { ApiError } from "api";
import { ChevronRight, CornerLeftUp, File, FileSymlink, Folder, Package } from "lucide-react";
import { Fragment } from "react";

import { BranchPicker } from "#/components/branch-picker";
import type { Entry, Tree } from "#/lib/queries";
import { branchesQuery, treeQuery } from "#/lib/queries";
import { cn } from "#/lib/utils";

interface Search {
  rev?: string;
  path?: string;
}

const text = (value: unknown) => (typeof value === "string" && value !== "" ? value : undefined);

const searchSchema = (search: Record<string, unknown>): Search => ({
  rev: text(search.rev),
  path: text(search.path),
});

export const Route = createFileRoute("/")({
  validateSearch: searchSchema,
  loaderDeps: ({ search }) => ({ rev: search.rev, path: search.path }),
  loader: async ({ context, deps, preload }) => {
    const tree = await context.queryClient
      .ensureQueryData(treeQuery(deps))
      .catch((error: unknown) => {
        if (error instanceof ApiError && error.status === 404) throw notFound();

        throw error;
      });

    if (!preload) context.branchStore.actions.select(tree.rev);
  },
  errorComponent: Failure,
  notFoundComponent: () => <Shell>no such file or directory on this branch</Shell>,
  component: Browser,
});

const ROW = "flex items-center gap-2.5 px-3 py-1.5 font-mono text-sm";

function Browser() {
  const deps = Route.useLoaderDeps();
  const { rev } = deps;
  const { data: tree } = useSuspenseQuery(treeQuery(deps));
  const { data: branches } = useSuspenseQuery(branchesQuery);

  const branchStore = Route.useRouteContext({ select: (context) => context.branchStore });
  const current = useStore(branchStore, (state) => state.current);

  return (
    <main className="mx-auto max-w-3xl px-6 py-10">
      <div className="mb-5">
        <BranchPicker branches={branches} current={current ?? tree.rev} />
      </div>

      <Crumbs rev={rev} path={tree.path} />
      <Listing tree={tree} rev={rev} />
    </main>
  );
}

function Crumbs({ rev, path }: { rev: string | undefined; path: string }) {
  const segments = path === "" ? [] : path.split("/");

  return (
    <nav className="mb-3 flex flex-wrap items-center gap-1 text-sm">
      <Link to="/" search={{ rev }} className="text-muted-foreground hover:text-foreground">
        root
      </Link>

      {segments.map((segment, index) => {
        const to = segments.slice(0, index + 1).join("/");

        return (
          <Fragment key={to}>
            <ChevronRight className="size-3.5 text-muted-foreground" />
            {index === segments.length - 1 ? (
              <span className="font-medium">{segment}</span>
            ) : (
              <Link
                to="/"
                search={{ rev, path: to }}
                className="text-muted-foreground hover:text-foreground"
              >
                {segment}
              </Link>
            )}
          </Fragment>
        );
      })}
    </nav>
  );
}

function Listing({ tree, rev }: { tree: Tree; rev: string | undefined }) {
  const parent = tree.path.split("/").slice(0, -1).join("/");

  return (
    <>
      <ul className="divide-y divide-border overflow-hidden rounded-lg border border-border">
        {tree.path !== "" && (
          <li>
            <Link
              to="/"
              search={{ rev, path: parent === "" ? undefined : parent }}
              className={cn(ROW, "text-muted-foreground transition-colors hover:bg-muted")}
            >
              <CornerLeftUp className="size-4" />
              ..
            </Link>
          </li>
        )}

        {tree.entries.map((entry) => (
          <Row key={entry.path} entry={entry} rev={rev} />
        ))}

        {tree.entries.length === 0 && (
          <li className="px-3 py-8 text-center text-sm text-muted-foreground">empty directory</li>
        )}
      </ul>

      {tree.has_more && (
        <p className="mt-3 text-xs text-muted-foreground">
          Showing {tree.entries.length}
          {tree.total === null ? "" : ` of ${tree.total}`} entries
        </p>
      )}
    </>
  );
}

function Row({ entry, rev }: { entry: Entry; rev: string | undefined }) {
  if (entry.kind === "directory") {
    return (
      <li>
        <Link
          to="/"
          search={{ rev, path: entry.path }}
          className={cn(ROW, "transition-colors hover:bg-muted")}
        >
          <Folder className="size-4 shrink-0 text-foreground" />
          {entry.name}
        </Link>
      </li>
    );
  }

  return (
    <li className={ROW}>
      <Icon kind={entry.kind} />
      <span className="truncate">{entry.name}</span>
      {entry.size !== null && (
        <span className="ml-auto shrink-0 text-xs text-muted-foreground tabular-nums">
          {bytes(entry.size)}
        </span>
      )}
    </li>
  );
}

function Icon({ kind }: { kind: Entry["kind"] }) {
  const className = "size-4 shrink-0 text-muted-foreground";

  if (kind === "symlink") return <FileSymlink className={className} />;
  if (kind === "submodule") return <Package className={className} />;

  return <File className={className} />;
}

function Failure({ error }: ErrorComponentProps) {
  return <Shell>{error instanceof Error ? error.message : String(error)}</Shell>;
}

function Shell({ children }: { children: React.ReactNode }) {
  return (
    <main className="mx-auto max-w-3xl px-6 py-10">
      <p className="mb-3 text-sm text-destructive">{children}</p>
      <Link to="/" className="text-sm underline underline-offset-4">
        back to root
      </Link>
    </main>
  );
}

const UNITS = ["B", "KB", "MB", "GB"];

function bytes(size: number) {
  let value = size;
  let unit = 0;

  while (value >= 1024 && unit < UNITS.length - 1) {
    value /= 1024;
    unit += 1;
  }

  return `${unit === 0 ? value : value.toFixed(1)} ${UNITS[unit]}`;
}
