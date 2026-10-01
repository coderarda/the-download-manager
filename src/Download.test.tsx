import React from "react";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const tauriCore = vi.hoisted(() => ({
  invoke: vi.fn(),
}));

const tauriEvent = vi.hoisted(() => ({
  listeners: new Map<string, (event: { payload: unknown }) => void>(),
  unlisten: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: tauriCore.invoke,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((event: string, callback: (event: { payload: unknown }) => void) => {
    tauriEvent.listeners.set(event, callback);
    return Promise.resolve(() => {
      tauriEvent.unlisten(event);
      tauriEvent.listeners.delete(event);
    });
  }),
}));

import Download from "./Download";

const createStatus = (overrides: Partial<DownloadStatus> = {}): DownloadStatus => ({
  item: {
    id: 7,
    url: "https://example.com/file.zip",
    title: "file.zip",
    filesize: 100,
  },
  paused: false,
  downloading: true,
  current_sz: 10,
  pausable: true,
  ...overrides,
});

describe("Download", () => {
  afterEach(() => {
    cleanup();
  });

  beforeEach(() => {
    tauriCore.invoke.mockReset();
    tauriCore.invoke.mockResolvedValue(undefined);
    tauriEvent.listeners.clear();
    tauriEvent.unlisten.mockClear();
  });

  it("pauses an active download", async () => {
    const status = createStatus({ paused: false });

    render(
      <table>
        <tbody>
          <Download status={status} />
        </tbody>
      </table>,
    );

    fireEvent.click(screen.getByRole("button", { name: /pause/i }));

    await waitFor(() => {
      expect(tauriCore.invoke).toHaveBeenCalledWith("pause_download", { id: status.item.id });
    });
    expect(screen.getByRole("button", { name: /resume/i })).toBeTruthy();
  });

  it("resumes a paused download", async () => {
    const status = createStatus({ paused: true });

    render(
      <table>
        <tbody>
          <Download status={status} />
        </tbody>
      </table>,
    );

    fireEvent.click(screen.getByRole("button", { name: /resume/i }));

    await waitFor(() => {
      expect(tauriCore.invoke).toHaveBeenCalledWith("resume", { download: status });
    });
    expect(screen.getByRole("button", { name: /pause/i })).toBeTruthy();
  });

  it("removes a download entry", async () => {
    const status = createStatus();

    render(
      <table>
        <tbody>
          <Download status={status} />
        </tbody>
      </table>,
    );

    const buttons = screen.getAllByRole("button");
    fireEvent.click(buttons[1]);

    await waitFor(() => {
      expect(tauriCore.invoke).toHaveBeenCalledWith("remove_download", { id: status.item.id });
    });
  });

  it("updates progress on download update events", async () => {
    const status = createStatus();

    render(
      <table>
        <tbody>
          <Download status={status} />
        </tbody>
      </table>,
    );

    await act(async () => {
      tauriEvent.listeners.get("ondownloadupdate")?.({
        payload: { id: status.item.id, chunk_size: 50 },
      });
    });

    expect(screen.getByText(/%50/)).toBeTruthy();
  });
});
