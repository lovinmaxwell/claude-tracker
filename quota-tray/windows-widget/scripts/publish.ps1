# Builds the portable Claude Widget: one self-contained exe that runs from any folder.
# No installer, no runtime to install, no admin rights; it only touches the current user's profile.
#
#   .\scripts\publish.ps1        # needs the .NET 10 SDK; the result is dist\ClaudeWidget.exe
#
# Or skip building: download ClaudeWidget.exe from the repo's "Claude Widget for Windows" release.
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')

dotnet test tests/ClaudeUsageCore.Tests -c Release
dotnet publish src/ClaudeWidget -c Release -r win-x64 -o dist
Remove-Item dist\*.pdb -ErrorAction SilentlyContinue

Write-Host ''
Write-Host "Built $((Resolve-Path dist\ClaudeWidget.exe).Path)"
Write-Host 'Double-click it to run. It starts at login by default; turn that off from the tray menu.'
