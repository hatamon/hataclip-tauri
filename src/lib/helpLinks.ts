export type HelpPiece =
  | { type: "text"; text: string }
  | { type: "link"; topic: string; label: string };

/** 本文の `:help 名前` をリンクにする。 */
export function splitHelpLinks(text: string): HelpPiece[] {
  const parts: HelpPiece[] = [];
  const pattern = /:help\s+(\S+)/g;
  let last = 0;
  for (const match of text.matchAll(pattern)) {
    const index = match.index ?? 0;
    if (index > last) {
      parts.push({ type: "text", text: text.slice(last, index) });
    }
    parts.push({ type: "link", topic: match[1], label: match[0] });
    last = index + match[0].length;
  }
  if (last < text.length) {
    parts.push({ type: "text", text: text.slice(last) });
  }
  if (parts.length === 0) {
    parts.push({ type: "text", text });
  }
  return parts;
}

export function sameHelpTopic(left: string | null | undefined, right: string | null): boolean {
  return (left ?? "") === (right ?? "");
}

/** クリックで進む。同じページは増やさない。戻ったあと別ページを開くと、先の分は捨てる。 */
export function pushHelpHistory(
  history: (string | null)[],
  cursor: number,
  topic: string | null,
): { history: (string | null)[]; cursor: number } {
  if (history.length > 0 && sameHelpTopic(history[cursor], topic)) {
    return { history, cursor };
  }
  const next = [...history.slice(0, cursor + 1), topic];
  return { history: next, cursor: next.length - 1 };
}
