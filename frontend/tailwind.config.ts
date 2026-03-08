import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        ink: "var(--ink)",
        sand: "var(--sand)",
        fog: "var(--fog)",
        accent: "var(--accent)",
        accentSoft: "var(--accent-soft)",
        line: "var(--line)",
      },
      boxShadow: {
        panel: "0 24px 60px rgba(15, 23, 42, 0.14)",
      },
      fontFamily: {
        sans: ["'IBM Plex Sans'", "system-ui", "sans-serif"],
        mono: ["'IBM Plex Mono'", "ui-monospace", "monospace"],
      },
    },
  },
  plugins: [],
};

export default config;
