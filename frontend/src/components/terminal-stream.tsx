"use client";

import { useEffect, useMemo, useState } from "react";

import { getSseUrl } from "@/lib/api";
import { subscribeToSse } from "@/lib/sse";
import { TerminalEvent } from "@/lib/types";

export function TerminalStream({
  taskId,
  initialEvents = [],
}: {
  taskId: string;
  initialEvents?: TerminalEvent[];
}) {
  const [events, setEvents] = useState<TerminalEvent[]>(initialEvents);

  useEffect(() => {
    const close = subscribeToSse(
      getSseUrl(`/api/tasks/${taskId}/terminal/stream`),
      (message) => {
        if (!message.data) {
          return;
        }
        try {
          const payload = JSON.parse(message.data) as TerminalEvent;
          setEvents((current) => {
            if (current.some((event) => event.id === payload.id)) {
              return current;
            }
            return [...current, payload];
          });
        } catch {
          return;
        }
      },
    );

    return close;
  }, [taskId]);

  const terminalText = useMemo(
    () => events.map((event) => event.chunk).join(""),
    [events],
  );

  return (
    <section className="surface overflow-hidden">
      <div className="surface-header">
        <h2 className="text-lg font-semibold text-ink">Live Terminal</h2>
      </div>
      <div className="surface-body">
        <pre className="max-h-[520px] overflow-auto rounded-lg bg-slate-950 p-4 whitespace-pre-wrap font-mono text-xs leading-6 text-emerald-200">
          {terminalText || "Waiting for terminal output..."}
        </pre>
      </div>
    </section>
  );
}
