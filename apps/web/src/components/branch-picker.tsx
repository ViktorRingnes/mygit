import { Combobox } from "@base-ui/react/combobox";
import { useRouter } from "@tanstack/react-router";
import { Check, ChevronsUpDown, GitBranch, Search } from "lucide-react";

import type { Branches } from "#/lib/queries";
import { cn } from "#/lib/utils";

interface BranchPickerProps {
  branches: Branches;
  current: string;
}

export function BranchPicker({ branches, current }: BranchPickerProps) {
  const router = useRouter();

  const searchFor = (name: string) => ({
    rev: name === branches.default_branch ? undefined : name,
  });

  return (
    <Combobox.Root
      items={branches.names}
      value={current}
      autoHighlight
      onValueChange={(name) => {
        if (name !== null) void router.navigate({ to: "/", search: searchFor(name) });
      }}
      onItemHighlighted={(name) => {
        if (name !== undefined) {
          router.preloadRoute({ to: "/", search: searchFor(name) }).catch(() => undefined);
        }
      }}
    >
      <Combobox.Label className="sr-only">Branch</Combobox.Label>

      <Combobox.Trigger
        className={cn(
          "inline-flex h-8 min-w-52 items-center gap-2 rounded-lg border border-border bg-background px-2.5",
          "text-sm font-medium transition-colors outline-none select-none",
          "hover:bg-muted focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50",
          "dark:border-input dark:bg-input/30 dark:hover:bg-input/50",
        )}
      >
        <GitBranch className="size-4 shrink-0 text-muted-foreground" />
        <span className="truncate font-mono text-[0.8rem]">
          <Combobox.Value />
        </span>
        <ChevronsUpDown className="ml-auto size-3.5 shrink-0 text-muted-foreground" />
      </Combobox.Trigger>

      <Combobox.Portal>
        <Combobox.Positioner align="start" sideOffset={6} className="z-50">
          <Combobox.Popup
            aria-label="Select a branch"
            className={cn(
              "w-[max(var(--anchor-width),18rem)] overflow-hidden rounded-lg border border-border",
              "bg-popover text-popover-foreground shadow-lg outline-none",
              "origin-[var(--transform-origin)] transition-[transform,opacity] duration-150",
              "data-ending-style:scale-95 data-ending-style:opacity-0",
              "data-starting-style:scale-95 data-starting-style:opacity-0",
            )}
          >
            <div className="flex items-center gap-2 border-b border-border px-3">
              <Search className="size-3.5 shrink-0 text-muted-foreground" />
              <Combobox.Input
                placeholder="Find a branch"
                className="h-9 w-full bg-transparent text-sm outline-none placeholder:text-muted-foreground"
              />
            </div>

            <Combobox.Empty>
              <p className="px-3 py-6 text-center text-sm text-muted-foreground">
                No branches match
              </p>
            </Combobox.Empty>

            <Combobox.List className="max-h-[min(20rem,var(--available-height))] overflow-y-auto overscroll-contain p-1">
              {(name: string) => (
                <Combobox.Item
                  key={name}
                  value={name}
                  className={cn(
                    "flex cursor-default items-center gap-2 rounded-md px-2 py-1.5 text-sm outline-none",
                    "data-highlighted:bg-muted data-highlighted:text-foreground",
                  )}
                >
                  <span className="flex size-3.5 shrink-0 items-center justify-center">
                    <Combobox.ItemIndicator>
                      <Check className="size-3.5" />
                    </Combobox.ItemIndicator>
                  </span>

                  <span className="truncate font-mono text-[0.8rem]">{name}</span>

                  <span className="ml-auto flex shrink-0 items-center gap-1">
                    {name === branches.default_branch && <Tag>default</Tag>}
                    {name === branches.head && <Tag>HEAD</Tag>}
                  </span>
                </Combobox.Item>
              )}
            </Combobox.List>
          </Combobox.Popup>
        </Combobox.Positioner>
      </Combobox.Portal>
    </Combobox.Root>
  );
}

function Tag({ children }: { children: React.ReactNode }) {
  return (
    <span className="rounded border border-border px-1 py-px font-mono text-[10px] text-muted-foreground">
      {children}
    </span>
  );
}
