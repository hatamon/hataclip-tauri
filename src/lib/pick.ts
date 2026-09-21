import { applyWhen } from "./when";

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

export type PickCatalog = { text: string; tags: string[] };

function pickTagName(spec: string): string | null {
  const name = argAfter(spec, "tag");
  return name && name.length > 0 ? name : null;
}

function tagOptions(name: string, catalog: PickCatalog[]): string[] {
  const options: string[] = [];
  for (const item of catalog) {
    if (!item.tags.includes(name) || item.text.length === 0) {
      continue;
    }
    if (!options.includes(item.text)) {
      options.push(item.text);
    }
  }
  return options;
}

export function pickSpecs(text: string, catalog: PickCatalog[] = [], app = ""): PickSpec[] {
  const source = applyWhen(text, app);
  const specs: PickSpec[] = [];
  let i = 0;
  while (i < source.length) {
    if (source[i] === "{" && source[i + 1] === "{") {
      const close = findClose(source, i + 2);
      if (close >= 0) {
        const inner = source.slice(i + 2, close).trim();
        const spec = argAfter(inner, "pick");
        if (spec && spec.length > 0 && !specs.some((entry) => entry.spec === spec)) {
          const tagName = pickTagName(spec);
          const options = tagName
            ? tagOptions(tagName, catalog)
            : spec
                .split(",")
                .map((part) => part.trim())
                .filter((part) => part.length > 0);
          if (options.length > 0) {
            specs.push({ spec, options });
          }
        }
        i = close + 2;
        continue;
      }
    }
    i += 1;
  }
  return specs;
}

export function uniquePickSpecs(
  items: { text: string }[],
  catalog: PickCatalog[] = [],
  app = "",
): PickSpec[] {
  const specs: PickSpec[] = [];
  for (const item of items) {
    for (const entry of pickSpecs(item.text, catalog, app)) {
      if (!specs.some((existing) => existing.spec === entry.spec)) {
        specs.push(entry);
      }
    }
  }
  return specs;
}
