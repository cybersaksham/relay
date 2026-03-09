"use client";

import { useEffect, useMemo, useState } from "react";

import { getSseUrl } from "@/lib/api";
import { subscribeToSse } from "@/lib/sse";
import { EnvironmentSyncEvent } from "@/lib/types";

export function EnvironmentSyncTerminal({
  environmentId,
}: {
  environmentId: string;
}) {
  const [events, setEvents] = useState<EnvironmentSyncEvent[]>([]);

  useEffect(() => {
    const close = subscribeToSse(
      getSseUrl(`/api/environments/${environmentId}/sync/stream`),
      (message) => {
        if (!message.data) {
          return;
        }
        try {
          const payload = JSON.parse(message.data) as EnvironmentSyncEvent;
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
  }, [environmentId]);

  const terminalText = useMemo(
    () => events.map((event) => event.chunk).join(""),
    [events],
  );

  return (
    <div className="surface overflow-hidden">
      <div className="surface-header">
        <h3 className="text-base font-semibold text-ink">Environment Sync Logs</h3>
      </div>
      <div className="surface-body">
        <pre className="max-h-[320px] overflow-auto whitespace-pre-wrap rounded-lg bg-slate-950 p-4 font-mono text-xs leading-6 text-emerald-200">
          {terminalText || "Waiting for sync logs..."}
        </pre>
      </div>
    </div>
  );
}
