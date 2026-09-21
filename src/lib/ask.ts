import { applyWhen } from "./when";

const ASK = /\{\{\s*ask[:\s]([^}]*)\}\}/g;

export function askNames(text: string, app = ""): string[] {
  const source = applyWhen(text, app);
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

export function uniqueAskNames(items: { text: string }[], app = ""): string[] {
  const names: string[] = [];
  for (const item of items) {
    for (const name of askNames(item.text, app)) {
      if (!names.includes(name)) {
        names.push(name);
      }
    }
  }
  return names;
}
