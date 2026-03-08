import type { Metadata } from "next";
import Link from "next/link";

import "@/app/globals.css";
import "@/styles/theme.css";

export const metadata: Metadata = {
  title: "Relay",
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
        <div className="panel-grid min-h-screen">
          <header className="border-b border-line bg-white/70 backdrop-blur">
            <div className="mx-auto flex max-w-7xl items-center justify-between px-6 py-5">
              <Link href="/" className="text-xl font-semibold tracking-[0.12em] text-accent">
                RELAY
              </Link>
              <nav className="flex items-center gap-6 text-sm text-slate-700">
                <Link href="/">Dashboard</Link>
                <Link href="/environments">Environments</Link>
              </nav>
            </div>
          </header>
          <main className="mx-auto w-full max-w-7xl px-6 py-8">{children}</main>
        </div>
      </body>
    </html>
  );
}
