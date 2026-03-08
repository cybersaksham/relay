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
    <div className="rounded-3xl border border-line bg-white/80 p-5 shadow-panel">
      <div className="mb-4 flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold">Transcript</h2>
          <p className="text-sm text-slate-600">Slack thread history resolved for the portal.</p>
        </div>
      </div>
      <div className="space-y-4">
        {messages.map((message) => (
          <article
            key={message.id}
            className={`max-w-3xl rounded-3xl px-4 py-3 ${
              message.direction === "outbound"
                ? "ml-auto bg-accent text-white"
                : "bg-sand text-ink"
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
    </div>
  );
}
