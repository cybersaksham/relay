"use client";

import { useEffect, useState } from "react";

import {
  getWorkspaceGitDiff,
  revertWorkspaceGitFile,
  stageWorkspaceGitFile,
} from "@/lib/api";
import { WorkspaceGitDiffResponse } from "@/lib/types";

export function WorkspaceGitDiffSheet({
  sessionId,
  workspacePath,
  isOpen,
  onClose,
}: {
  sessionId: string;
  workspacePath: string;
  isOpen: boolean;
  onClose: () => void;
}) {
  const [diff, setDiff] = useState<WorkspaceGitDiffResponse | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pendingAction, setPendingAction] = useState<string | null>(null);

  useEffect(() => {
    if (!isOpen) {
      return;
    }

    void loadDiff();
  }, [isOpen]);

  async function loadDiff() {
    setIsLoading(true);
    setError(null);
    try {
      setDiff(await getWorkspaceGitDiff(sessionId));
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : "Failed to load git diff.");
    } finally {
      setIsLoading(false);
    }
  }

  async function handleStage(path: string) {
    setPendingAction(`stage:${path}`);
    setError(null);
    try {
      setDiff(await stageWorkspaceGitFile(sessionId, path));
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : "Failed to stage file.");
    } finally {
      setPendingAction(null);
    }
  }

  async function handleRevert(path: string) {
    setPendingAction(`revert:${path}`);
    setError(null);
    try {
      setDiff(await revertWorkspaceGitFile(sessionId, path));
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : "Failed to revert file.");
    } finally {
      setPendingAction(null);
    }
  }

  if (!isOpen) {
    return null;
  }

  const isEmpty = diff && (diff.available === false || diff.files.length === 0);

  return (
    <div
      className="fixed inset-0 z-50 bg-slate-950/45"
      onClick={onClose}
      role="presentation"
    >
      <div className="flex h-full justify-end">
        <section
          className="flex h-full w-full max-w-3xl flex-col overflow-hidden border-l border-line bg-white shadow-2xl"
          onClick={(event) => event.stopPropagation()}
          aria-label="Workspace git diff"
        >
          <div className="flex flex-wrap items-center justify-between gap-3 border-b border-line px-5 py-4">
            <div className="min-w-0">
              <h2 className="text-lg font-semibold text-ink">Workspace Git Diff</h2>
              <p className="mt-1 truncate font-mono text-xs text-slate-500">{workspacePath}</p>
            </div>
            <div className="flex items-center gap-2">
              <button
                type="button"
                onClick={() => void loadDiff()}
                disabled={isLoading || pendingAction !== null}
                className="rounded-full border border-line px-3 py-1.5 text-sm font-medium text-slate-700 transition hover:bg-fog disabled:cursor-not-allowed disabled:opacity-60"
              >
                Refresh
              </button>
              <button
                type="button"
                onClick={onClose}
                className="rounded-full border border-line px-3 py-1.5 text-sm font-medium text-slate-700 transition hover:bg-fog"
              >
                Close
              </button>
            </div>
          </div>

          <div className="min-h-0 flex-1 overflow-y-auto bg-slate-50 p-4">
            {error ? (
              <div className="rounded-2xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
                {error}
              </div>
            ) : null}

            {isLoading && !diff ? (
              <div className="flex h-full items-center justify-center rounded-2xl border border-dashed border-line bg-white p-10 text-sm text-slate-500">
                Loading workspace diff...
              </div>
            ) : null}

            {!isLoading && isEmpty ? (
              <div className="flex h-full items-center justify-center rounded-2xl border border-dashed border-line bg-white p-10 text-center">
                <div className="max-w-sm space-y-2">
                  <div className="text-base font-semibold text-ink">No workspace changes</div>
                  <p className="text-sm text-slate-500">
                    {diff?.available === false
                      ? diff.reason ?? "Git is not configured for this workspace."
                      : "No changed files were found in this workspace."}
                  </p>
                </div>
              </div>
            ) : null}

            {diff?.files.length ? (
              <div className="space-y-4">
                {diff.files.map((file) => {
                  const isStagePending = pendingAction === `stage:${file.path}`;
                  const isRevertPending = pendingAction === `revert:${file.path}`;
                  return (
                    <article
                      key={file.path}
                      className="overflow-hidden rounded-2xl border border-line bg-white shadow-sm"
                    >
                      <div className="flex flex-wrap items-center justify-between gap-3 border-b border-line px-4 py-3">
                        <div className="min-w-0">
                          <div className="truncate font-mono text-sm text-ink">{file.path}</div>
                          <div className="mt-1 text-xs uppercase tracking-[0.16em] text-slate-500">
                            {file.status}
                          </div>
                        </div>
                        <div className="flex flex-wrap items-center gap-2">
                          <button
                            type="button"
                            onClick={() => void handleRevert(file.path)}
                            disabled={pendingAction !== null}
                            className="rounded-full border border-red-200 bg-red-50 px-3 py-1.5 text-sm font-medium text-red-700 transition hover:bg-red-100 disabled:cursor-not-allowed disabled:opacity-60"
                          >
                            {isRevertPending ? "Reverting..." : "Revert"}
                          </button>
                          <button
                            type="button"
                            onClick={() => void handleStage(file.path)}
                            disabled={pendingAction !== null || !file.can_stage}
                            className="rounded-full border border-line px-3 py-1.5 text-sm font-medium text-slate-700 transition hover:bg-fog disabled:cursor-not-allowed disabled:opacity-60"
                          >
                            {isStagePending ? "Staging..." : file.staged && !file.can_stage ? "Staged" : "Stage"}
                          </button>
                        </div>
                      </div>
                      <pre className="overflow-x-auto bg-slate-950 px-4 py-3 text-xs leading-6 text-slate-200">
                        <code>
                          {file.diff ? (
                            <DiffLines diff={file.diff} />
                          ) : (
                            <span className="text-slate-400">No textual diff available.</span>
                          )}
                        </code>
                      </pre>
                    </article>
                  );
                })}
              </div>
            ) : null}
          </div>
        </section>
      </div>
    </div>
  );
}

function DiffLines({ diff }: { diff: string }) {
  return diff.split("\n").map((line, index, lines) => {
    const key = `${index}-${line}`;
    const className =
      line.startsWith("+") && !line.startsWith("+++")
        ? "text-emerald-300"
        : line.startsWith("-") && !line.startsWith("---")
          ? "text-rose-300"
          : line.startsWith("@@")
            ? "text-sky-300"
            : "text-slate-200";

    return (
      <span key={key} className={`block whitespace-pre ${className}`}>
        {line}
        {index < lines.length - 1 ? "\n" : ""}
      </span>
    );
  });
}
