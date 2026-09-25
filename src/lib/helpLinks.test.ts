import { describe, expect, it } from "vitest";
import { pushHelpHistory, splitHelpLinks } from "./helpLinks";

describe("splitHelpLinks", () => {
  it("turns :help names into links", () => {
    expect(splitHelpLinks("詳しくは :help quote と :help ctrl8")).toEqual([
      { type: "text", text: "詳しくは " },
      { type: "link", topic: "quote", label: ":help quote" },
      { type: "text", text: " と " },
      { type: "link", topic: "ctrl8", label: ":help ctrl8" },
    ]);
  });
});

describe("pushHelpHistory", () => {
  it("drops the forward pages after a new jump", () => {
    const first = pushHelpHistory([], 0, null);
    const second = pushHelpHistory(first.history, first.cursor, "quote");
    const back = { history: second.history, cursor: second.cursor - 1 };
    const branched = pushHelpHistory(back.history, back.cursor, "dd");
    expect(branched.history).toEqual([null, "dd"]);
    expect(branched.cursor).toBe(1);
    const again = pushHelpHistory(branched.history, branched.cursor, "dd");
    expect(again.cursor).toBe(1);
    expect(again.history).toEqual([null, "dd"]);
  });
});
