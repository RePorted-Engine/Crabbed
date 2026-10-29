# Windows version of fetch-artifacts.sh: the newest successful run's artifacts for a branch into
# .\ci-artifacts\<run-id>\, copied to .\ci-artifacts\latest\<workflow>\. Needs the GitHub CLI, signed in.
#   .\scripts\fetch-artifacts.ps1 [-Branch main] [-Workflows linux,ios-sim]
param(
  [string]$Branch = (git rev-parse --abbrev-ref HEAD),
  [string[]]$Workflows = @("linux", "ios-sim"),
  [string]$Repo = "heyskylo/horizon-ios"
)
$ErrorActionPreference = "Stop"
Set-Location (git rev-parse --show-toplevel)
New-Item -ItemType Directory -Force ci-artifacts\latest | Out-Null
foreach ($wf in $Workflows) {
  $id = gh run list -R $Repo --workflow "$wf.yml" --branch $Branch --status success --limit 1 --json databaseId --jq '.[0].databaseId // empty'
  if (-not $id) { Write-Host "${wf}: no successful run on $Branch"; continue }
  $dir = "ci-artifacts\$id"
  if (-not (Test-Path $dir)) { gh run download $id -R $Repo -D $dir }
  Get-ChildItem $dir -Recurse -Filter *.tar.gz | ForEach-Object { tar -xzf $_.FullName -C $_.DirectoryName }
  $latest = "ci-artifacts\latest\$wf"
  if (Test-Path $latest) { Remove-Item -Recurse -Force $latest }
  Copy-Item -Recurse $dir $latest
  Write-Host "${wf}: run $id -> $latest"
}
