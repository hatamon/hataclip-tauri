param(
  [string]$ImageName = "hataclip-tauri-builder-linux",
  [string]$DockerfilePath = "docker/linux/Dockerfile"
)

$RepoRoot = (Resolve-Path "$PSScriptRoot\..").Path

docker build -t $ImageName -f (Join-Path $RepoRoot $DockerfilePath) $RepoRoot

# Windowsバイナリは `--no-bundle` 前提で、Linuxからのクロスコンパイルで生成します。
docker run --rm `
  -v "${RepoRoot}:/work" `
  -w /work `
  $ImageName `
  bash -lc "sed -i 's/\r$//' ./scripts/build-in-docker.sh && TAURI_TARGET=x86_64-pc-windows-gnu TAURI_NO_BUNDLE=1 bash ./scripts/build-in-docker.sh"

