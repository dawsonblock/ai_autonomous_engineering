import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { StatusBadge } from "@/components/status-badge";

describe("StatusBadge", () => {
  it("renders trust labels visibly", () => {
    render(<StatusBadge value="degraded" />);
    expect(screen.getByText("degraded")).toBeInTheDocument();
  });
});
