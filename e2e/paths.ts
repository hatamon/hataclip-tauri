// Docker（Linux）とホスト（Windows）で同じ場所を指す2つの見え方を作る。
// hataclip-gui.exe にはホストのパス（\ 区切り）を渡し、
// この Vitest からはコンテナのパス（/ 区切り）で読む。

function requireEnv(name: string): string {
  const value = process.env[name];
  if (!value || value.trim() === "") {
    throw new Error(`${name} が無い。e2e.env を用意して --env-file で渡す`);
  }
  return value;
}

/** WinAppDriver が実際に起動する exe に渡す、ホスト側から見た絶対パス。 */
export function hostPath(...segments: string[]): string {
  const root = requireEnv("E2E_HOST_ROOT").replace(/[\\/]+$/, "");
  return [root, ...segments].join("\\");
}

/** このコンテナから同じ場所を読むときの絶対パス。 */
export function containerPath(...segments: string[]): string {
  const root = (process.env.E2E_CONTAINER_ROOT ?? "/work/e2e-workspace").replace(/\/+$/, "");
  return [root, ...segments].join("/");
}

/** ホストでビルドした hataclip-gui.exe の Windows パス。 */
export function hataclipGuiPath(): string {
  return requireEnv("HATACLIP_GUI");
}

/** WinAppDriver が動いているホスト名。無ければ E2E は skip する。 */
export function winAppDriverHost(): string | undefined {
  return process.env.WINAPPDRIVER_HOST;
}
