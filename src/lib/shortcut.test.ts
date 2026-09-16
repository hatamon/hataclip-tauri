import { describe, expect, it } from "vitest";
import { fromEvent, toLabel } from "./shortcut";

function keydown(code: string, modifiers: Partial<Record<"ctrl" | "alt" | "shift" | "meta", true>> = {}) {
  return {
    code,
    ctrlKey: modifiers.ctrl ?? false,
    altKey: modifiers.alt ?? false,
    shiftKey: modifiers.shift ?? false,
    metaKey: modifiers.meta ?? false,
  };
}

describe("fromEvent", () => {
  it("builds a shortcut from modifiers and a key", () => {
    expect(fromEvent(keydown("Digit4", { ctrl: true }))).toBe("Control+Digit4");
    expect(fromEvent(keydown("KeyY", { ctrl: true, shift: true }))).toBe("Control+Shift+KeyY");
  });

  it("returns null without a modifier", () => {
    expect(fromEvent(keydown("Digit4"))).toBeNull();
  });

  it("returns null while only a modifier is held", () => {
    expect(fromEvent(keydown("ControlLeft", { ctrl: true }))).toBeNull();
  });
});

describe("toLabel", () => {
  it("renders a readable combination", () => {
    expect(toLabel("Control+Digit4")).toBe("Ctrl + 4");
    expect(toLabel("Control+Shift+KeyY")).toBe("Ctrl + Shift + Y");
  });

  it("keeps unknown keys as they are", () => {
    expect(toLabel("Alt+F12")).toBe("Alt + F12");
  });
});
