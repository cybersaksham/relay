import type { Metadata } from "next";

import "@/app/globals.css";
import "@/styles/theme.css";
import { AppShellNav } from "@/components/app-shell-nav";

export const metadata: Metadata = {
  title: "Relay Local",
  description: "Relay bot management portal",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>
        <div className="min-h-screen bg-fog lg:grid lg:grid-cols-[248px_minmax(0,1fr)]">
          <aside className="border-b border-line bg-accent px-5 py-6 text-white lg:border-b-0 lg:border-r">
            <div className="space-y-8">
              <div className="space-y-2">
                <p className="text-xl font-semibold">Relay Local</p>
                <p className="max-w-[18rem] text-sm leading-6 text-slate-300">
                  Codex-like local control plane for Slack-driven task orchestration.
                </p>
              </div>
              <AppShellNav />
            </div>
          </aside>
          <main className="min-w-0 px-4 py-6 sm:px-6 lg:px-10 lg:py-8">
            <div className="mx-auto max-w-6xl">{children}</div>
          </main>
        </div>
      </body>
    </html>
  );
}
