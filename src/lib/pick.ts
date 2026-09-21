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

export type PickSpec = { spec: string; options: string[] };

export function pickSpecs(text: string): PickSpec[] {
  const specs: PickSpec[] = [];
  let i = 0;
  while (i < text.length) {
    if (text[i] === "{" && text[i + 1] === "{") {
      const close = findClose(text, i + 2);
      if (close >= 0) {
        const inner = text.slice(i + 2, close).trim();
        const spec = argAfter(inner, "pick");
        if (spec && spec.length > 0 && !specs.some((entry) => entry.spec === spec)) {
          const options = spec
            .split(",")
            .map((part) => part.trim())
            .filter((part) => part.length > 0);
          specs.push({ spec, options });
        }
        i = close + 2;
        continue;
      }
    }
    i += 1;
  }
  return specs;
}

export function uniquePickSpecs(items: { text: string }[]): PickSpec[] {
  const specs: PickSpec[] = [];
  for (const item of items) {
    for (const entry of pickSpecs(item.text)) {
      if (!specs.some((existing) => existing.spec === entry.spec)) {
        specs.push(entry);
      }
    }
  }
  return specs;
}

