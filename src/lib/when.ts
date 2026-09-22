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

function isIdent(name: string): boolean {
  return /^[A-Za-z_][A-Za-z0-9_]*$/.test(name);
}

/** `"` で括った値。`\"` は `"`、`\t` はタブ、`\n` は改行。 */
export function unquoteDouble(raw: string): string | null {
  const text = raw.trim();
  if (!text.startsWith('"')) {
    return null;
  }
  let out = "";
  let escaped = false;
  for (let i = 1; i < text.length; i += 1) {
    const ch = text[i];
    if (escaped) {
      out += ch === "t" ? "\t" : ch === "n" ? "\n" : ch;
      escaped = false;
      continue;
    }
    if (ch === "\\") {
      escaped = true;
      continue;
    }
    if (ch === '"') {
      if (text.slice(i + 1).trim().length > 0) {
        return null;
      }
      return out;
    }
    out += ch;
  }
  return null;
}

export type WhenEnv = {
  app?: string;
  focus?: string;
  vars?: Record<string, string>;
};

type WhenKind =
  | { kind: "app"; names: string[] }
  | { kind: "var"; name: string; value: string }
  | { kind: "focus"; name: string }
  | { kind: "fallback" };

function asEnv(env: string | WhenEnv | undefined): Required<WhenEnv> {
  if (typeof env === "string" || env === undefined) {
    return { app: env ?? "", focus: "", vars: {} };
  }
  return { app: env.app ?? "", focus: env.focus ?? "", vars: env.vars ?? {} };
}

function whenKind(token: string): WhenKind | null {
  if (token === "when") {
    return { kind: "fallback" };
  }
  const arg = argAfter(token, "when");
  if (arg === null) {
    return null;
  }
  if (arg.startsWith("app:")) {
    const names = arg
      .slice(4)
      .split(",")
      .map((name) => name.trim())
      .filter((name) => name.length > 0);
    return names.length > 0 ? { kind: "app", names } : null;
  }
  if (arg.startsWith("var:")) {
    const rest = arg.slice(4).trim();
    const split = rest.indexOf(":");
    if (split < 0) {
      return null;
    }
    const name = rest.slice(0, split).trim();
    if (!isIdent(name)) {
      return null;
    }
    const value = unquoteDouble(rest.slice(split + 1));
    return value === null ? null : { kind: "var", name, value };
  }
  if (arg.startsWith("focus:")) {
    const name = arg.slice(6).trim();
    return name.length > 0 ? { kind: "focus", name } : null;
  }
  return null;
}

function whenHits(kind: WhenKind, env: Required<WhenEnv>): boolean {
  if (kind.kind === "app") {
    return env.app.length > 0 && kind.names.some((name) => name.toLowerCase() === env.app.toLowerCase());
  }
  if (kind.kind === "var") {
    return env.vars[kind.name] === kind.value;
  }
  if (kind.kind === "focus") {
    return env.focus.length > 0 && kind.name.toLowerCase() === env.focus.toLowerCase();
  }
  return false;
}

/** `{{when app:}}` `{{when var:}}` `{{when focus:}}` `{{when}}`。`{{when chrome}}` は枝にしない。 */
export function applyWhen(text: string, env: string | WhenEnv = ""): string {
  const current = asEnv(env);
  const marks: { start: number; end: number; kind: WhenKind }[] = [];
  let i = 0;
  while (i < text.length) {
    if (text[i] === "{" && text[i + 1] === "{") {
      const close = findClose(text, i + 2);
      if (close >= 0) {
        const kind = whenKind(text.slice(i + 2, close).trim());
        if (kind) {
          marks.push({ start: i, end: close + 2, kind });
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
  for (let index = 0; index < marks.length; index += 1) {
    const body = text.slice(marks[index].end, index + 1 < marks.length ? marks[index + 1].start : text.length);
    if (marks[index].kind.kind === "fallback") {
      fallback = body;
    } else if (chosen === null && whenHits(marks[index].kind, current)) {
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
