param(
  [string]$ImageName = "hataclip-tauri-builder-linux",
  [string]$DockerfilePath = "docker/linux/Dockerfile"
)

$RepoRoot = (Resolve-Path "$PSScriptRoot\..").Path

docker build -t $ImageName -f (Join-Path $RepoRoot $DockerfilePath) $RepoRoot

# Mount repo into the container so the build sees current source.
docker run --rm `
  -v "${RepoRoot}:/work" `
  -w /work `
  $ImageName `
  bash -lc "sed -i 's/\r$//' ./scripts/build-in-docker.sh && TAURI_TARGET=x86_64-unknown-linux-gnu TAURI_NO_BUNDLE=1 bash ./scripts/build-in-docker.sh"

