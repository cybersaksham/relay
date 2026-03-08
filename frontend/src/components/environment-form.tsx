"use client";

import { FormEvent, useState } from "react";

import { EnvironmentSummary } from "@/lib/types";

export interface EnvironmentFormValues {
  name: string;
  slug: string;
  git_ssh_url: string;
  default_branch: string;
  aliases: string[];
  enabled: boolean;
}

interface EnvironmentFormProps {
  title: string;
  description: string;
  submitLabel: string;
  submittingLabel: string;
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
};

export function environmentToFormValues(environment: EnvironmentSummary): EnvironmentFormValues {
  return {
    name: environment.name,
    slug: environment.slug,
    git_ssh_url: environment.git_ssh_url,
    default_branch: environment.default_branch,
    aliases: parseAliases(environment.aliases),
    enabled: environment.enabled,
  };
}

export function parseAliases(value: string): string[] {
  try {
    const parsed = JSON.parse(value);
    return Array.isArray(parsed) ? parsed.filter((item) => typeof item === "string") : [];
  } catch {
    return value
      .split(",")
      .map((item) => item.trim())
      .filter(Boolean);
  }
}

export function EnvironmentForm({
  title,
  description,
  submitLabel,
  submittingLabel,
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

    const form = new FormData(event.currentTarget);
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
      });
      if (!initialValues) {
        event.currentTarget.reset();
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
      className="rounded-3xl border border-line bg-white/80 p-6 shadow-panel backdrop-blur"
    >
      <div className="mb-5 flex items-start justify-between gap-4">
        <div>
          <h2 className="text-xl font-semibold">{title}</h2>
          <p className="text-sm text-slate-600">{description}</p>
        </div>
      </div>
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
      </div>
      {error ? <p className="mt-4 text-sm text-red-700">{error}</p> : null}
      <div className="mt-5 flex flex-wrap gap-3">
        <button
          type="submit"
          disabled={pending}
          className="rounded-full bg-accent px-5 py-2.5 text-sm font-medium text-white transition hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50"
        >
          {pending ? submittingLabel : submitLabel}
        </button>
        {onCancel ? (
          <button
            type="button"
            onClick={onCancel}
            className="rounded-full border border-line px-5 py-2.5 text-sm font-medium text-slate-700 transition hover:bg-fog"
          >
            Cancel
          </button>
        ) : null}
      </div>
    </form>
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
        className="rounded-2xl border border-line bg-sand px-4 py-3 text-sm outline-none ring-0 transition focus:border-accent"
        name={name}
        placeholder={placeholder}
        defaultValue={defaultValue}
        required={required}
      />
    </label>
  );
}
