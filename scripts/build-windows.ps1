#!/usr/bin/env pwsh
#Requires -Version 7
$ErrorActionPreference = "Stop"
Set-Location (Split-Path -Parent $PSScriptRoot)
docker compose run --rm windows-build
