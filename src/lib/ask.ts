import { applyWhen, type WhenEnv } from "./when";

const ASK = /\{\{\s*ask[:\s]([^}]*)\}\}/g;

export function askNames(text: string, env: string | WhenEnv = ""): string[] {
  const source = applyWhen(text, env);
  const names: string[] = [];
  ASK.lastIndex = 0;
  let match: RegExpExecArray | null;
  while ((match = ASK.exec(source)) !== null) {
    const name = match[1].trim();
    if (!names.includes(name)) {
      names.push(name);
    }
  }
  return names;
}

export function uniqueAskNames(items: { text: string }[], env: string | WhenEnv = ""): string[] {
  const names: string[] = [];
  for (const item of items) {
    for (const name of askNames(item.text, env)) {
      if (!names.includes(name)) {
        names.push(name);
      }
    }
  }
  return names;
}
