#!/usr/bin/env pwsh
#Requires -Version 7
$ErrorActionPreference = "Stop"
Set-Location (Split-Path -Parent $PSScriptRoot)
Get-Process hataclip, hataclip-gui, hataclip-cli -ErrorAction SilentlyContinue | Stop-Process -Force
docker compose run --rm windows-build
