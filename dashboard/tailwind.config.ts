import type { Config } from "tailwindcss";

export default {
  darkMode: ["class"],
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        ink: {
          50: "#f8f8f5",
          100: "#efefe8",
          200: "#dddccd",
          300: "#c2c0ad",
          400: "#9d9b84",
          500: "#7d7b67",
          600: "#656351",
          700: "#4f4d40",
          800: "#3f3d33",
          900: "#2a2923",
        },
        signal: {
          blue: "#1677ff",
          green: "#109868",
          amber: "#c88114",
          red: "#c53a2f",
        },
      },
      boxShadow: {
        panel: "0 20px 60px rgba(24, 32, 43, 0.12)",
      },
      fontFamily: {
        sans: ["'IBM Plex Sans'", "sans-serif"],
        mono: ["'IBM Plex Mono'", "monospace"],
      },
    },
  },
  plugins: [],
} satisfies Config;
