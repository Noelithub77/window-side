#!/usr/bin/env bash
# Read-only parser, simulated environment guards and real driver detection in a running QEMU guest.
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
script=$(mktemp --suffix=.ps1)
trap 'rm -f -- "$script"' EXIT
{
  cat <<'PS'
$ErrorActionPreference = 'Stop'
$reports = @()
PS
  for name in first-boot.ps1 display-startup.ps1 install-looking-glass.ps1; do
    encoded=$(base64 -w0 "$root/assets/guest/$name")
    printf '\n$source=[Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('\''%s'\''))\n' "$encoded"
    cat <<'PS'
$tokens=$null; $errors=$null
[void][System.Management.Automation.Language.Parser]::ParseInput($source,[ref]$tokens,[ref]$errors)
if ($errors.Count -gt 0) { throw ($errors | Out-String) }
PS
    printf '$reports+=@{name='\''%s'\'';syntaxErrors=0}\n' "$name"
    if [[ "$name" == first-boot.ps1 ]]; then printf '$readiness=$source\n'; fi
    if [[ "$name" != display-startup.ps1 ]]; then
      cat <<'PS'
$guard = & {
  param($script, $installer)
  function Get-CimInstance { [pscustomobject]@{Manufacturer='Physical vendor';Model='Laptop'} }
  $rejected=$false
  try {
    if ($installer) { & ([scriptblock]::Create($script)) -Installer 'not-a-file.exe' -Refresh '60' }
    else { & ([scriptblock]::Create($script)) -Apply }
  } catch {
    if ($_.Exception.Message -notmatch 'QEMU Windows VM') { throw }
    $rejected=$true
  }
  if (-not $rejected) { throw 'The physical-machine guard did not reject provisioning' }
  $rejected
PS
      if [[ "$name" == install-looking-glass.ps1 ]]; then
        printf '} $source $true\n'
      else
        printf '} $source $false\n'
      fi
    fi
  done
  cat <<'PS'
$guest = & ([scriptblock]::Create($readiness)) | ConvertFrom-Json
@{environment='window-side-libvirt';scripts=$reports;physicalGuardsRejected=$true;applied=$false;readiness=$guest} |
  ConvertTo-Json -Depth 5 -Compress
PS
} > "$script"
"$root/target/debug/ws" exec --file "$script" --timeout 120 --json
