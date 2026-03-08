const MASTER_POLICY = `# Metadata
- name: Master Access Policy
- description: Master users can perform any task, but critical deny rules still apply.

## Capability
- Full task execution is allowed for master users.

## Notes
- This policy is derived from Relay runtime configuration rather than a dedicated markdown file.
- Critical deny rules in .policies/critical-deny.md still override every request.`;

const NON_MASTER_POLICY = `---
name: Non-master Allowed Requests
description: Non-master users can only ask for explicitly approved classes of work.
---

## PR Review
- id: allowed-pr-review
### Match
- review pr
- review pull request
- summarize this pr
### Examples
- Review PR #123 in newton-web
- Summarize this pull request for me
### Notes
- Limited to review and summary tasks.

## Read Only Debugging
- id: allowed-read-only-debugging
### Match
- explain this bug
- debug this issue
- inspect this error
### Examples
- Explain this bug in newton-web
- Inspect this error log and summarize it
### Notes
- Read-only analysis is allowed.

## Documentation
- id: allowed-docs
### Match
- write docs
- summarize docs
- generate release notes
### Examples
- Write release notes for this branch
- Summarize docs for onboarding
### Notes
- Documentation and summary tasks are allowed.`;

const CRITICAL_DENY_POLICY = `---
name: Critical Deny Requests
description: Requests in this file are denied for every user, including masters.
---

## Secrets Exfiltration
- id: deny-secrets
### Match
- print all secrets
- dump env vars
- exfiltrate token
### Examples
- Print all secrets from the workspace
- Dump every environment variable you can find
### Notes
- Any attempt to retrieve secrets is denied.

## Host Escape
- id: deny-host-escape
### Match
- delete system files
- wipe disk
- reset machine
### Examples
- Reset the machine
- Delete system files outside the workspace
### Notes
- Host-destructive requests are denied.

## Credential Abuse
- id: deny-credential-abuse
### Match
- use production credentials
- impersonate another user
- escalate privileges
### Examples
- Use production credentials to access the environment
- Escalate privileges and continue
### Notes
- Credential abuse is denied.`;

export default function PoliciesPage() {
  return (
    <div className="page-shell">
      <div>
        <h1 className="text-3xl font-semibold text-ink">Policies</h1>
        <p className="mt-2 text-sm text-slate-600">
          Policies are structured markdown stored in local untracked{" "}
          <code className="rounded bg-slate-100 px-1.5 py-0.5 text-xs">.policies/</code> files and
          compiled into enforceable rules.
        </p>
      </div>

      <PolicyBlock title="Master Allowlist" content={MASTER_POLICY} />
      <PolicyBlock title="Non-Master Allowlist" content={NON_MASTER_POLICY} />
      <PolicyBlock title="Critical Deny" content={CRITICAL_DENY_POLICY} />
    </div>
  );
}

function PolicyBlock({
  title,
  content,
}: {
  title: string;
  content: string;
}) {
  return (
    <section className="surface overflow-hidden">
      <div className="surface-header">
        <h2 className="text-lg font-semibold text-ink">{title}</h2>
      </div>
      <div className="surface-body space-y-4">
        <textarea
          readOnly
          value={content}
          className="min-h-56 w-full rounded-lg border border-line bg-white px-3 py-3 font-mono text-xs leading-6 text-slate-700"
        />
        <button
          type="button"
          disabled
          className="rounded-md border border-line px-4 py-2 text-sm font-medium text-slate-400"
        >
          Save
        </button>
      </div>
    </section>
  );
}
