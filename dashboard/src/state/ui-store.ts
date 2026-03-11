import { create } from "zustand";

type Theme = "light" | "dark";

type UiState = {
  theme: Theme;
  recentLaunches: Array<Record<string, unknown>>;
  setTheme: (theme: Theme) => void;
  rememberLaunch: (launch: Record<string, unknown>) => void;
};

const storedTheme = (typeof window !== "undefined" && window.localStorage.getItem("aae-theme")) as Theme | null;

export const useUiStore = create<UiState>((set) => ({
  theme: storedTheme === "dark" ? "dark" : "light",
  recentLaunches: [],
  setTheme: (theme) => {
    if (typeof window !== "undefined") {
      window.localStorage.setItem("aae-theme", theme);
    }
    set({ theme });
  },
  rememberLaunch: (launch) =>
    set((state) => ({
      recentLaunches: [launch, ...state.recentLaunches].slice(0, 6),
    })),
}));
