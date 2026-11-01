# The per-user half of the installer (installer/jismoonshot.iss runs it as the user who started
# Setup, not elevated): adds JIS Moonshot to the Korean language's keyboards, or removes it, as
# tools/install.ps1 does. Other input methods (JIS Korean included) and the rest of the language
# list are left alone.
#   keyboards.ps1                     add JIS Moonshot (and Korean, if it isn't in the list)
#   keyboards.ps1 -AcceptPredictive   also store that predictive typing was accepted on Setup's
#                                     notice page (PredictiveAccepted, docs/moonshot.md §6)
#   keyboards.ps1 -Remove             take it out again
param(
    [switch] $Remove,
    [switch] $AcceptPredictive
)
$ErrorActionPreference = 'Stop'
$tip = '0412:{EC381A55-0AD1-42DF-BBAB-AF4F34F90C1C}{42BB462F-CA66-4FD0-9289-951EAE5C4BB3}'

$list = Get-WinUserLanguageList
$korean = $null
foreach ($l in $list.ToArray()) { if ($l.LanguageTag -eq 'ko') { $korean = $l } }
if ($Remove) {
    if ($korean -and $korean.InputMethodTips -contains $tip) {
        [void]$korean.InputMethodTips.Remove($tip)
        Set-WinUserLanguageList $list -Force
    }
    exit 0
}
if (-not $korean) {
    $korean = (New-WinUserLanguageList 'ko')[0]
    $korean.InputMethodTips.Clear()
    $list.Add($korean)
}
if ($korean.InputMethodTips -notcontains $tip) {
    $korean.InputMethodTips.Add($tip)
    Set-WinUserLanguageList $list -Force
}

# Accepted on Setup's notice page: the TIP types from the first key. The value is the notice's
# version (settings.h kConsentVersion); a higher one already there stays.
if ($AcceptPredictive) {
    $key = 'HKCU:\Software\JIS Moonshot'
    if (-not (Test-Path $key)) { New-Item $key | Out-Null }
    $now = (Get-ItemProperty $key -Name PredictiveAccepted -ErrorAction SilentlyContinue).PredictiveAccepted
    if ($null -eq $now -or $now -lt 1) { Set-ItemProperty $key PredictiveAccepted 1 -Type DWord }
}
exit 0
