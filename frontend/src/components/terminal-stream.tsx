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
    <section className="rounded-3xl border border-slate-800 bg-slate-950 p-5 shadow-panel">
      <div className="mb-4 flex items-center justify-between text-slate-300">
        <div>
          <h2 className="text-lg font-semibold text-white">Live Terminal</h2>
          <p className="text-xs uppercase tracking-[0.2em] text-slate-400">
            Read only Codex activity
          </p>
        </div>
      </div>
      <pre className="max-h-[520px] overflow-auto whitespace-pre-wrap font-mono text-xs leading-6 text-emerald-200">
        {terminalText || "Waiting for terminal output..."}
      </pre>
    </section>
  );
}
