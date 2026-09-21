function findClose(text: string, start: number): number {
  let depth = 1;
  let i = start;
  while (i + 1 < text.length) {
    if (text[i] === "{" && text[i + 1] === "{") {
      depth += 1;
      i += 2;
      continue;
    }
    if (text[i] === "}" && text[i + 1] === "}") {
      depth -= 1;
      if (depth === 0) {
        return i;
      }
      i += 2;
      continue;
    }
    i += 1;
  }
  return -1;
}

function argAfter(inner: string, name: string): string | null {
  if (!inner.startsWith(name)) {
    return null;
  }
  const rest = inner.slice(name.length);
  if (rest.startsWith(":")) {
    return rest.slice(1).trim();
  }
  if (rest.length > 0 && /\s/.test(rest[0])) {
    return rest.trim();
  }
  return null;
}

function parseWhenApps(inner: string): string[] | null {
  const token = inner.trim();
  if (token === "when") {
    return [];
  }
  const arg = argAfter(token, "when");
  if (arg === null) {
    return null;
  }
  return arg
    .split(/[,\s]+/)
    .map((name) => name.trim())
    .filter((name) => name.length > 0);
}

/** `{{when chrome}}…{{when}}既定`。先頭の `{{when` より前は常に残す。 */
export function applyWhen(text: string, app: string): string {
  const marks: { start: number; end: number; apps: string[] }[] = [];
  let i = 0;
  while (i < text.length) {
    if (text[i] === "{" && text[i + 1] === "{") {
      const close = findClose(text, i + 2);
      if (close >= 0) {
        const inner = text.slice(i + 2, close);
        const apps = parseWhenApps(inner);
        if (apps !== null) {
          marks.push({ start: i, end: close + 2, apps });
        }
        i = close + 2;
        continue;
      }
    }
    i += 1;
  }
  if (marks.length === 0) {
    return text;
  }
  const prefix = text.slice(0, marks[0].start);
  let chosen: string | null = null;
  let fallback: string | null = null;
  const current = app.toLowerCase();
  for (let index = 0; index < marks.length; index += 1) {
    const body = text.slice(
      marks[index].end,
      index + 1 < marks.length ? marks[index + 1].start : text.length,
    );
    if (marks[index].apps.length === 0) {
      fallback = body;
    } else if (
      chosen === null &&
      marks[index].apps.some((name) => name.toLowerCase() === current)
    ) {
      chosen = body;
    }
  }
  return `${prefix}${chosen ?? fallback ?? ""}`;
}

export function contextApp(context: string | null | undefined): string {
  if (!context) {
    return "";
  }
  return context.split("|")[0] ?? "";
}
