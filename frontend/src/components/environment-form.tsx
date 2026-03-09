"use client";

import { FormEvent, useState } from "react";

import { parseAliases } from "@/lib/environments";
import { EnvironmentSummary } from "@/lib/types";

export interface EnvironmentFormValues {
  name: string;
  slug: string;
  git_ssh_url: string;
  default_branch: string;
  aliases: string[];
  enabled: boolean;
  source_setup_script: string;
  workspace_setup_script: string;
}

interface EnvironmentFormProps {
  title: string;
  description: string;
  submitLabel: string;
  submittingLabel: string;
  hideHeader?: boolean;
  initialValues?: Partial<EnvironmentFormValues>;
  onSubmit: (values: EnvironmentFormValues) => Promise<void>;
  onCancel?: () => void;
}

const DEFAULT_VALUES: EnvironmentFormValues = {
  name: "",
  slug: "",
  git_ssh_url: "",
  default_branch: "",
  aliases: [],
  enabled: true,
  source_setup_script: "",
  workspace_setup_script: "",
};

export function environmentToFormValues(environment: EnvironmentSummary): EnvironmentFormValues {
  return {
    name: environment.name,
    slug: environment.slug,
    git_ssh_url: environment.git_ssh_url,
    default_branch: environment.default_branch,
    aliases: parseAliases(environment.aliases),
    enabled: environment.enabled,
    source_setup_script: environment.source_setup_script ?? "",
    workspace_setup_script: environment.workspace_setup_script ?? "",
  };
}

export function EnvironmentForm({
  title,
  description,
  submitLabel,
  submittingLabel,
  hideHeader,
  initialValues,
  onSubmit,
  onCancel,
}: EnvironmentFormProps) {
  const values = {
    ...DEFAULT_VALUES,
    ...initialValues,
  };
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setPending(true);
    setError(null);

    const formElement = event.currentTarget;
    const form = new FormData(formElement);
    const aliases = String(form.get("aliases") ?? "")
      .split(",")
      .map((item) => item.trim())
      .filter(Boolean);

    try {
      await onSubmit({
        name: String(form.get("name") ?? ""),
        slug: String(form.get("slug") ?? ""),
        git_ssh_url: String(form.get("git_ssh_url") ?? ""),
        default_branch: String(form.get("default_branch") ?? ""),
        aliases,
        enabled: true,
        source_setup_script: String(form.get("source_setup_script") ?? "").trim(),
        workspace_setup_script: String(form.get("workspace_setup_script") ?? "").trim(),
      });
      if (!initialValues) {
        formElement.reset();
      }
    } catch (submitError) {
      setError(submitError instanceof Error ? submitError.message : "Failed to save environment");
    } finally {
      setPending(false);
    }
  }

  return (
    <form
      onSubmit={handleSubmit}
      className="space-y-4"
    >
      {hideHeader ? null : (
        <div className="flex items-start justify-between gap-4">
          <div>
            <h2 className="text-lg font-semibold text-ink">{title}</h2>
            <p className="text-sm text-slate-600">{description}</p>
          </div>
        </div>
      )}
      <div className="grid gap-4 md:grid-cols-2">
        <Field label="Name" name="name" placeholder="Newton Web" defaultValue={values.name} required />
        <Field
          label="Slug"
          name="slug"
          placeholder="newton-web"
          defaultValue={values.slug}
          required
        />
        <Field
          label="Git SSH URL"
          name="git_ssh_url"
          placeholder="git@github.com:org/repo.git"
          defaultValue={values.git_ssh_url}
          required
        />
        <Field
          label="Default Branch"
          name="default_branch"
          placeholder="main"
          defaultValue={values.default_branch}
          required
        />
        <Field
          label="Aliases"
          name="aliases"
          placeholder="newton, web, frontend"
          defaultValue={values.aliases.join(", ")}
          className="md:col-span-2"
        />
        <TextAreaField
          label="Source Setup Script"
          name="source_setup_script"
          placeholder="npm ci"
          defaultValue={values.source_setup_script}
          className="md:col-span-2"
        />
        <TextAreaField
          label="Workspace Setup Script"
          name="workspace_setup_script"
          placeholder="git fetch --depth 1 origin master && git reset --hard origin/master && npm ci"
          defaultValue={values.workspace_setup_script}
          className="md:col-span-2"
        />
      </div>
      {error ? <p className="text-sm text-red-700">{error}</p> : null}
      <div className="flex flex-wrap gap-3">
        <button
          type="submit"
          disabled={pending}
          className="rounded-md bg-slate-900 px-4 py-2 text-sm font-medium text-white transition hover:bg-slate-800 disabled:cursor-not-allowed disabled:opacity-50"
        >
          {pending ? submittingLabel : submitLabel}
        </button>
        {onCancel ? (
          <button
            type="button"
            onClick={onCancel}
            className="rounded-md border border-line px-4 py-2 text-sm font-medium text-slate-700 transition hover:bg-fog"
          >
            Cancel
          </button>
        ) : null}
      </div>
    </form>
  );
}

function TextAreaField({
  label,
  name,
  placeholder,
  defaultValue,
  className,
}: {
  label: string;
  name: string;
  placeholder?: string;
  defaultValue?: string;
  className?: string;
}) {
  return (
    <label className={`flex flex-col gap-2 ${className ?? ""}`}>
      <span className="text-sm font-medium">{label}</span>
      <textarea
        className="min-h-24 w-full rounded-lg border border-line bg-white px-3 py-2 font-mono text-sm outline-none ring-0 transition focus:border-slate-400"
        name={name}
        placeholder={placeholder}
        defaultValue={defaultValue}
      />
    </label>
  );
}

function Field({
  label,
  name,
  placeholder,
  defaultValue,
  required,
  className,
}: {
  label: string;
  name: string;
  placeholder?: string;
  defaultValue?: string;
  required?: boolean;
  className?: string;
}) {
  return (
    <label className={`flex flex-col gap-2 ${className ?? ""}`}>
      <span className="text-sm font-medium">{label}</span>
      <input
        className="w-full rounded-lg border border-line bg-white px-3 py-2 text-sm outline-none ring-0 transition focus:border-slate-400"
        name={name}
        placeholder={placeholder}
        defaultValue={defaultValue}
        required={required}
      />
    </label>
  );
}
