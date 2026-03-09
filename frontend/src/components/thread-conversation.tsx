"use client";

import { useRouter } from "next/navigation";
import { useState } from "react";

import { cancelTask } from "@/lib/api";
import { formatUtcTimestamp } from "@/lib/format";
import { TaskMessage, TaskSummary } from "@/lib/types";

import { TerminalStream } from "@/components/terminal-stream";

function parseResolvedText(payload: string): string {
  try {
    const parsed = JSON.parse(payload) as { text?: string };
    return parsed.text ?? "";
  } catch {
    return payload;
  }
}

interface RunConversation {
  run: TaskSummary;
  inbound: TaskMessage[];
  outbound: TaskMessage[];
}

export function ThreadConversation({
  sessionId,
  runs,
  messages,
}: {
  sessionId: string;
  runs: TaskSummary[];
  messages: TaskMessage[];
}) {
  const router = useRouter();
  const [cancellingRunId, setCancellingRunId] = useState<string | null>(null);
  const [cancelError, setCancelError] = useState<string | null>(null);
  const sortedRuns = [...runs].sort(
    (left, right) => new Date(left.started_at).getTime() - new Date(right.started_at).getTime(),
  );
  const sortedMessages = [...messages].sort(
    (left, right) => new Date(left.created_at).getTime() - new Date(right.created_at).getTime(),
  );
  const runConversations: RunConversation[] = sortedRuns.map((run) => ({
    run,
    inbound: [],
    outbound: [],
  }));
  const runIndexById = new Map(runConversations.map((item, index) => [item.run.id, index]));
  const ungroupedMessages: TaskMessage[] = [];

  for (const message of sortedMessages) {
    if (message.task_run_id) {
      const conversationIndex = runIndexById.get(message.task_run_id);
      if (conversationIndex !== undefined) {
        if (message.direction === "inbound") {
          runConversations[conversationIndex].inbound.push(message);
        } else {
          runConversations[conversationIndex].outbound.push(message);
        }
        continue;
      }
    }

    if (message.task_run_id === null && message.direction === "inbound") {
      const inferredConversation = runConversations.find(
        (conversation) =>
          new Date(conversation.run.started_at).getTime() >=
          new Date(message.created_at).getTime(),
      );
      if (inferredConversation) {
        inferredConversation.inbound.push(message);
        continue;
      }
    }

    ungroupedMessages.push(message);
  }

  async function handleCancel(runId: string) {
    setCancelError(null);
    setCancellingRunId(runId);

    try {
      await cancelTask(sessionId);
      router.refresh();
    } catch (error) {
      setCancelError(error instanceof Error ? error.message : "Failed to cancel request.");
    } finally {
      setCancellingRunId(null);
    }
  }

  return (
    <section className="surface overflow-hidden">
      <div className="surface-header">
        <h2 className="text-lg font-semibold text-ink">Thread Conversation</h2>
      </div>
      <div className="surface-body space-y-6">
        {cancelError ? (
          <div className="rounded-xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
            {cancelError}
          </div>
        ) : null}
        {runConversations.map((conversation, index) => (
          <article
            key={conversation.run.id}
            className="rounded-2xl border border-line bg-white p-5 shadow-sm"
          >
            <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
              <div>
                <div className="text-xs uppercase tracking-[0.18em] text-slate-500">
                  Request {index + 1}
                </div>
                <div className="mt-1 text-sm text-slate-600">
                  Started {formatUtcTimestamp(conversation.run.started_at)}
                </div>
              </div>
              <div className="flex flex-wrap gap-2 text-xs">
                <span className="rounded-full border border-line px-2 py-1 font-medium uppercase tracking-[0.12em] text-slate-600">
                  {conversation.run.status.replaceAll("_", " ")}
                </span>
                <span className="rounded-full border border-line px-2 py-1 font-medium text-slate-600">
                  {conversation.run.workflow_name ?? "Generic run"}
                </span>
                {isCancelableStatus(conversation.run.status) ? (
                  <button
                    type="button"
                    onClick={() => void handleCancel(conversation.run.id)}
                    disabled={cancellingRunId !== null}
                    className="rounded-full border border-red-200 bg-red-50 px-3 py-1 font-medium text-red-700 transition hover:bg-red-100 disabled:cursor-not-allowed disabled:opacity-70"
                  >
                    {cancellingRunId === conversation.run.id
                      ? "Cancelling..."
                      : "Cancel Request"}
                  </button>
                ) : null}
              </div>
            </div>

            <div className="space-y-4">
              {conversation.inbound.map((message) => (
                <MessageBubble key={message.id} message={message} />
              ))}

              <TerminalStream
                taskId={conversation.run.id}
                title="Processing"
                emptyCopy="Waiting for terminal output for this request..."
                maxHeightClass="max-h-[280px]"
              />

              {conversation.outbound.map((message) => (
                <MessageBubble key={message.id} message={message} />
              ))}
            </div>
          </article>
        ))}

        {ungroupedMessages.length > 0 ? (
          <div className="space-y-3">
            <div className="text-xs uppercase tracking-[0.18em] text-slate-500">
              Thread Messages
            </div>
            {ungroupedMessages.map((message) => (
              <MessageBubble key={message.id} message={message} />
            ))}
          </div>
        ) : null}
      </div>
    </section>
  );
}

function isCancelableStatus(status: string): boolean {
  return status === "queued" || status === "running" || status === "waiting_for_reply";
}

function MessageBubble({ message }: { message: TaskMessage }) {
  const isRelay = message.direction === "outbound";

  return (
    <article
      className={`rounded-2xl border px-4 py-3 ${
        isRelay
          ? "border-slate-200 bg-slate-900 text-white"
          : "border-line bg-slate-50 text-ink"
      }`}
    >
      <div className="mb-2 text-xs uppercase tracking-[0.2em] opacity-70">
        {isRelay ? "Relay" : message.slack_user_id ?? "User"}
      </div>
      <p className="whitespace-pre-wrap text-sm leading-6">
        {parseResolvedText(message.resolved_payload)}
      </p>
    </article>
  );
}
