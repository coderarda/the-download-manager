import React from "react";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const tauriEvent = vi.hoisted(() => ({
  unlisten: vi.fn(),
  listen: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: tauriEvent.listen.mockImplementation(
    (_event: string, _callback: (event: { payload: unknown }) => void) => {
      return Promise.resolve(() => {
        tauriEvent.unlisten();
      });
    },
  ),
}));

import { AppProvider, useAppContext } from "./AppContext";

describe("AppContext", () => {
  afterEach(() => {
    cleanup();
  });

  beforeEach(() => {
    tauriEvent.unlisten.mockClear();
    tauriEvent.listen.mockClear();
  });

  it("throws when hook is used outside provider", () => {
    const OutsideComponent = () => {
      useAppContext();
      return null;
    };

    expect(() => render(<OutsideComponent />)).toThrow(
      "useAppContext must be used within an AppProvider",
    );
  });

  it("provides empty downloads by default", () => {
    const Consumer = () => {
      const { downloads } = useAppContext();
      return <div data-testid="downloads-count">{downloads.length}</div>;
    };

    render(
      <AppProvider>
        <Consumer />
      </AppProvider>,
    );

    expect(screen.getByTestId("downloads-count").textContent).toBe("0");
  });

  it("registers and unregisters the download listener", async () => {
    const Child = () => <div>child</div>;

    const { unmount } = render(
      <AppProvider>
        <Child />
      </AppProvider>,
    );

    expect(tauriEvent.listen).toHaveBeenCalledWith(
      "downloadfirstupdate",
      expect.any(Function),
    );

    unmount();

    await waitFor(() => {
      expect(tauriEvent.unlisten).toHaveBeenCalledTimes(1);
    });
  });
});
