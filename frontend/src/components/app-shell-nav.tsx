"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

const NAV_ITEMS = [
  { href: "/", label: "Dashboard" },
  { href: "/environments", label: "Environments" },
  { href: "/tasks", label: "Tasks" },
  { href: "/chats", label: "Chats" },
  { href: "/policies", label: "Policies" },
  { href: "/bans", label: "Bans" },
  { href: "/slack", label: "Slack" },
  { href: "/pull-requests", label: "Pull Requests" },
];

export function AppShellNav() {
  const pathname = usePathname();

  return (
    <nav className="space-y-1">
      {NAV_ITEMS.map((item) => {
        const active =
          item.href === "/" ? pathname === item.href : pathname.startsWith(item.href);

        return (
          <Link
            key={item.href}
            href={item.href}
            className={`flex items-center rounded-lg px-3 py-2 text-sm font-medium transition ${
              active
                ? "bg-white/12 text-white"
                : "text-slate-300 hover:bg-white/6 hover:text-white"
            }`}
          >
            {item.label}
          </Link>
        );
      })}
    </nav>
  );
}
