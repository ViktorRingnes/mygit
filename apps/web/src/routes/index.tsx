import { Button } from "#/components/ui/button";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/")({ component: Home });

function Home() {
  return (
    <div className="p-8">
      <p>hey</p>
      <Button>hey</Button>
    </div>
  );
}
