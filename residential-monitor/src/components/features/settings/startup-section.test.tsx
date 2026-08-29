import { createRef } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";
import type { AutostartRequestState } from "../../../hooks/autostart-request";
import {
  StartupConfirmation,
  StartupSection,
  commitStartupConfirmation,
  scheduleStartupFocus,
  startupToggleAction
} from "./startup-section";

const readyOff: AutostartRequestState = {
  enabled: false,
  loaded: true,
  loading: false,
  saving: false,
  errorZh: null
};

describe("StartupSection", () => {
  it("routes enable through confirmation and disable directly", () => {
    expect(startupToggleAction(false, true, false)).toBe("confirm-enable");
    expect(startupToggleAction(true, false, false)).toBe("disable");
    expect(startupToggleAction(false, true, true)).toBe("none");
    expect(startupToggleAction(true, true, false)).toBe("none");
  });

  it("cancel does not write and confirmation requests enable once", () => {
    const writes: boolean[] = [];
    commitStartupConfirmation(false, (enabled) => writes.push(enabled));
    expect(writes).toEqual([]);
    commitStartupConfirmation(true, (enabled) => writes.push(enabled));
    expect(writes).toEqual([true]);
  });

  it("renders a named checked/disabled switch from confirmed state", () => {
    const loading = renderToStaticMarkup(
      <StartupSection
        locale="zh"
        state={{ ...readyOff, loaded: false, loading: true }}
        onRefresh={() => undefined}
        onSetEnabled={() => undefined}
      />
    );
    expect(loading).toContain("正在读取 Windows 登录自启动状态");
    expect(loading).toContain("disabled");
    expect(loading).toContain("登录 Windows 后自动启动 ResiWatch");

    const saving = renderToStaticMarkup(
      <StartupSection
        locale="zh"
        state={{ ...readyOff, saving: true }}
        onRefresh={() => undefined}
        onSetEnabled={() => undefined}
      />
    );
    expect(saving).toContain("正在保存并回读系统状态");
    expect(saving).toContain("disabled");

    const enabled = renderToStaticMarkup(
      <StartupSection
        locale="en"
        state={{ ...readyOff, enabled: true }}
        onRefresh={() => undefined}
        onSetEnabled={() => undefined}
      />
    );
    expect(enabled).toContain('aria-checked="true"');
    expect(enabled).toContain("Enabled");
  });

  it("confirmation is an accessible bilingual alertdialog", () => {
    const zh = renderToStaticMarkup(
      <StartupConfirmation
        locale="zh"
        titleId="title"
        descriptionId="description"
        confirmButtonRef={createRef<HTMLButtonElement>()}
        onConfirm={() => undefined}
        onCancel={() => undefined}
      />
    );
    expect(zh).toContain('role="alertdialog"');
    expect(zh).toContain('aria-labelledby="title"');
    expect(zh).toContain('aria-describedby="description"');
    expect(zh).toContain("确认开启");
    expect(zh).toContain("--background");

    const en = renderToStaticMarkup(
      <StartupConfirmation
        locale="en"
        titleId="title"
        descriptionId="description"
        confirmButtonRef={createRef<HTMLButtonElement>()}
        onConfirm={() => undefined}
        onCancel={() => undefined}
      />
    );
    expect(en).toContain("Confirm enable");
    expect(en).toContain("system tray");
  });

  it("supports Escape cancellation and returns focus to the switch", () => {
    const onCancel = vi.fn();
    const confirmation = StartupConfirmation({
      locale: "zh",
      titleId: "title",
      descriptionId: "description",
      confirmButtonRef: createRef<HTMLButtonElement>(),
      onConfirm: () => undefined,
      onCancel
    });
    const preventDefault = vi.fn();
    confirmation.props.onKeyDown({ key: "Escape", preventDefault });
    expect(preventDefault).toHaveBeenCalledOnce();
    expect(onCancel).toHaveBeenCalledOnce();

    confirmation.props.onKeyDown({ key: "Enter", preventDefault });
    expect(preventDefault).toHaveBeenCalledOnce();
    expect(onCancel).toHaveBeenCalledOnce();

    const focus = vi.fn();
    const scheduled: Array<() => void> = [];
    scheduleStartupFocus(() => ({ focus }), (task) => scheduled.push(task));
    expect(focus).not.toHaveBeenCalled();
    expect(scheduled).toHaveLength(1);
    scheduled[0]?.();
    expect(focus).toHaveBeenCalledOnce();
  });

  it("keeps the last confirmed value visible when an error is retryable", () => {
    const html = renderToStaticMarkup(
      <StartupSection
        locale="zh"
        state={{ ...readyOff, enabled: true, errorZh: "读取失败" }}
        onRefresh={() => undefined}
        onSetEnabled={() => undefined}
      />
    );
    expect(html).toContain('aria-checked="true"');
    expect(html).toContain('role="alert"');
    expect(html).toContain("重试");
  });
});
