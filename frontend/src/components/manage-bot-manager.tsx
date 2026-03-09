"use client";

import { FormEvent, useMemo, useState } from "react";

import { deleteSlackMessage, lookupSlackMessage, updateSlackMessage } from "@/lib/api";
import { ManagedSlackMessage } from "@/lib/types";

export function ManageBotManager() {
  const [permalink, setPermalink] = useState("");
  const [message, setMessage] = useState<ManagedSlackMessage | null>(null);
  const [draftText, setDraftText] = useState("");
  const [lookupPending, setLookupPending] = useState(false);
  const [savePending, setSavePending] = useState(false);
  const [deletePending, setDeletePending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);

  const messageLocation = useMemo(() => {
    if (!message) {
      return null;
    }

    return message.thread_ts && message.thread_ts !== message.ts
      ? `Reply in thread ${message.thread_ts}`
      : "Top-level message";
  }, [message]);

  async function handleLookup(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setLookupPending(true);
    setError(null);
    setSuccess(null);
    setConfirmDelete(false);
    try {
      const fetched = await lookupSlackMessage(permalink);
      setMessage(fetched);
      setDraftText(fetched.raw_text);
    } catch (lookupError) {
      setMessage(null);
      setDraftText("");
      setError(
        lookupError instanceof Error ? lookupError.message : "Failed to fetch Slack message.",
      );
    } finally {
      setLookupPending(false);
    }
  }

  async function handleSave() {
    if (!message) {
      return;
    }

    setSavePending(true);
    setError(null);
    setSuccess(null);
    try {
      const updated = await updateSlackMessage({
        channel_id: message.channel_id,
        ts: message.ts,
        thread_ts: message.thread_ts,
        text: draftText,
      });
      setMessage(updated);
      setDraftText(updated.raw_text);
      setSuccess("Slack message updated.");
    } catch (saveError) {
      setError(saveError instanceof Error ? saveError.message : "Failed to update Slack message.");
    } finally {
      setSavePending(false);
    }
  }

  async function handleDelete() {
    if (!message) {
      return;
    }

    setDeletePending(true);
    setError(null);
    setSuccess(null);
    try {
      await deleteSlackMessage({
        channel_id: message.channel_id,
        ts: message.ts,
        thread_ts: message.thread_ts,
      });
      setMessage(null);
      setDraftText("");
      setPermalink("");
      setConfirmDelete(false);
      setSuccess("Slack message deleted.");
    } catch (deleteError) {
      setError(deleteError instanceof Error ? deleteError.message : "Failed to delete Slack message.");
    } finally {
      setDeletePending(false);
    }
  }

  return (
    <>
      <div className="page-shell">
        <div>
          <h1 className="text-3xl font-semibold text-ink">Manage Bot</h1>
          <p className="mt-2 max-w-3xl text-sm text-slate-600">
            Paste a Slack message permalink to fetch a message posted by the bot configured through
            <code className="mx-1 rounded bg-slate-100 px-1.5 py-0.5 text-xs">SLACK_BOT_TOKEN</code>
            and then edit or delete it.
          </p>
        </div>

        <section className="surface overflow-hidden">
          <div className="surface-header">
            <h2 className="text-lg font-semibold text-ink">Fetch Bot Message</h2>
          </div>
          <form onSubmit={handleLookup} className="surface-body space-y-4">
            <label className="block space-y-2">
              <span className="text-sm font-medium text-slate-700">Slack message link</span>
              <input
                value={permalink}
                onChange={(event) => setPermalink(event.target.value)}
                className="w-full rounded-lg border border-line bg-white px-3 py-2 text-sm"
                placeholder="https://workspace.slack.com/archives/C12345678/p1736451111222233"
              />
            </label>
            <button
              type="submit"
              disabled={lookupPending}
              className="rounded-md bg-slate-900 px-4 py-2 text-sm font-medium text-white transition hover:bg-slate-800 disabled:cursor-not-allowed disabled:opacity-50"
            >
              {lookupPending ? "Fetching..." : "Fetch Message"}
            </button>
            {error ? <p className="text-sm text-red-700">{error}</p> : null}
            {success ? <p className="text-sm text-emerald-700">{success}</p> : null}
          </form>
        </section>

        {message ? (
          <>
            <section className="surface overflow-hidden">
              <div className="surface-header">
                <h2 className="text-lg font-semibold text-ink">Message Details</h2>
              </div>
              <div className="surface-body space-y-3 text-sm text-slate-700">
                <p>
                  Channel:{" "}
                  <code className="rounded bg-slate-100 px-1.5 py-0.5 text-xs">
                    {message.channel_id}
                  </code>
                </p>
                <p>
                  Timestamp:{" "}
                  <code className="rounded bg-slate-100 px-1.5 py-0.5 text-xs">{message.ts}</code>
                </p>
                {messageLocation ? <p>{messageLocation}</p> : null}
              </div>
            </section>

            <section className="surface overflow-hidden">
              <div className="surface-header">
                <h2 className="text-lg font-semibold text-ink">Edit Message</h2>
              </div>
              <div className="surface-body space-y-4">
                <label className="block space-y-2">
                  <span className="text-sm font-medium text-slate-700">Message text</span>
                  <textarea
                    value={draftText}
                    onChange={(event) => setDraftText(event.target.value)}
                    className="min-h-40 w-full rounded-lg border border-line bg-white px-3 py-2 text-sm"
                  />
                </label>
                <div className="flex flex-wrap gap-3">
                  <button
                    type="button"
                    onClick={handleSave}
                    disabled={savePending || draftText.trim().length === 0}
                    className="rounded-md bg-slate-900 px-4 py-2 text-sm font-medium text-white transition hover:bg-slate-800 disabled:cursor-not-allowed disabled:opacity-50"
                  >
                    {savePending ? "Saving..." : "Save Changes"}
                  </button>
                  <button
                    type="button"
                    onClick={() => {
                      setError(null);
                      setSuccess(null);
                      setConfirmDelete(true);
                    }}
                    className="rounded-md bg-red-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-red-700"
                  >
                    Delete Message
                  </button>
                </div>
              </div>
            </section>
          </>
        ) : null}
      </div>

      {confirmDelete && message ? (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/45 px-4 py-8">
          <div className="w-full max-w-xl">
            <div className="mb-3 flex items-center justify-between text-white">
              <h2 className="text-lg font-semibold">Delete Slack Message</h2>
              <button
                type="button"
                onClick={() => setConfirmDelete(false)}
                className="rounded-md border border-white/20 px-3 py-1.5 text-sm transition hover:bg-white/10"
              >
                Close
              </button>
            </div>
            <div className="surface p-6">
              <p className="text-sm text-slate-600">
                Delete the bot message at{" "}
                <code className="rounded bg-slate-100 px-1.5 py-0.5 text-xs">{message.ts}</code> in{" "}
                <code className="rounded bg-slate-100 px-1.5 py-0.5 text-xs">
                  {message.channel_id}
                </code>
                . This cannot be undone.
              </p>
              <div className="mt-6 flex flex-wrap gap-3">
                <button
                  type="button"
                  onClick={handleDelete}
                  disabled={deletePending}
                  className="rounded-md bg-red-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-red-700 disabled:cursor-not-allowed disabled:opacity-50"
                >
                  {deletePending ? "Deleting..." : "Delete Permanently"}
                </button>
                <button
                  type="button"
                  onClick={() => setConfirmDelete(false)}
                  className="rounded-md border border-line px-4 py-2 text-sm font-medium text-slate-700 transition hover:bg-fog"
                >
                  Cancel
                </button>
              </div>
            </div>
          </div>
        </div>
      ) : null}
    </>
  );
}
