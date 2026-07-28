# Redirect to the actual install script
$scriptUrl = "https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/scripts/install.ps1"
Write-Host "Downloading from $scriptUrl..."
iex (iwr -Uri $scriptUrl -UseBasicParsing).Content
