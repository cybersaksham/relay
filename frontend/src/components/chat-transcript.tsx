import { TaskMessage } from "@/lib/types";

function parseResolvedText(payload: string): string {
  try {
    const parsed = JSON.parse(payload) as { text?: string };
    return parsed.text ?? "";
  } catch {
    return payload;
  }
}

export function ChatTranscript({ messages }: { messages: TaskMessage[] }) {
  return (
    <section className="surface overflow-hidden">
      <div className="surface-header">
        <h2 className="text-lg font-semibold text-ink">Transcript</h2>
      </div>
      <div className="surface-body space-y-3">
        {messages.map((message) => (
          <article
            key={message.id}
            className={`rounded-lg border px-4 py-3 ${
              message.direction === "outbound"
                ? "border-slate-200 bg-slate-900 text-white"
                : "border-line bg-slate-50 text-ink"
            }`}
          >
            <div className="mb-2 text-xs uppercase tracking-[0.2em] opacity-70">
              {message.direction === "outbound" ? "Relay" : message.slack_user_id ?? "User"}
            </div>
            <p className="whitespace-pre-wrap text-sm leading-6">
              {parseResolvedText(message.resolved_payload)}
            </p>
          </article>
        ))}
      </div>
    </section>
  );
}
