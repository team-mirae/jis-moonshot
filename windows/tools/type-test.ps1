# Types key sequences through the installed TIP into jk_testpad and checks the text: a classic
# Edit control and a RichEdit through the IMM compatibility layer, and a RichEdit on TSF itself. The testpad is the foreground window while it runs: don't touch
# the keyboard. Settings that change what's typed (Mode, predictions in jk_testpad.exe) are set for
# the run and restored after, predictive typing is accepted meanwhile (docs/moonshot.md §6), and
# Learn from My Choices is off: the test's picks aren't the user's.
#   tools/type-test.ps1                 all cases, both controls
#   tools/type-test.ps1 -Target rich    one control
param(
    [ValidateSet('edit', 'rich', 'tsf')] [string[]] $Target = @('edit', 'rich', 'tsf'),
    [string] $Filter = ''
)
$ErrorActionPreference = 'Stop'
$pad = Resolve-Path "$PSScriptRoot\..\build\x64\jk_testpad.exe"
$key = 'HKCU:\Software\JIS Moonshot'
if (-not (Test-Path $key)) { New-Item $key | Out-Null }
# Predictions are always on: the cases without them exclude jk_testpad.exe, as the menu's
# "Predictions in <program>" does.
$appKey = "$key\Apps\jk_testpad.exe"
if (-not (Test-Path $appKey)) { New-Item $appKey -Force | Out-Null }

# name, Mode (0 A, 11 JIS Kana), Predictions, keys, expected text in the control[, EnglishMode:
# ABC's layout id, 'unset' for the default (JIS English), 'windows' for Windows' own]
# Cases end committed: classic controls (edit, rich) hold only the first character of an open
# composition (docs/troubleshooting/windows-tsf-notes.md).
$cases = @(
    @('한글 안녕 (Space commits)', 0, 0, 'KeyG KeyK KeyS KeyR KeyM KeyF Space KeyD KeyK KeyS KeyS KeyU KeyD Space', '한글 안녕 '),
    @('Enter commits, new line', 0, 0, 'KeyG KeyK KeyS Enter KeyG KeyK Space', "한`r`n하 "),
    @('arrow commits, then moves', 0, 0, 'KeyG KeyK KeyS ArrowLeft KeyG KeyK Space', '하 한'),
    @('Tab commits, then passes', 0, 0, 'KeyG KeyK KeyS Tab', "한`t"),
    @('Tab picks a prediction', 0, 1, 'KeyH KeyE KeyL KeyL KeyO Tab Tab Space', 'Hello '),
    @('Shift+F8 list: ↓ then 1-9 page pick', 0, 0, 'KeyG KeyK KeyR KeyR KeyY S-F8 ArrowDown Digit1 Space', '學校 '),
    @('implied ㅇ: ㅏ → 아', 0, 0, 'KeyK Space', '아 '),
    @('number row 1 → ㅃ, 0 → ㅘ', 0, 0, 'Digit1 KeyK Digit0 Space', '빠와 '),
    @('Shift+T is ㅅ, 5 is ㅆ: 잇 있', 0, 0, 'KeyD KeyL S-KeyT Space KeyD KeyL Digit5 Space', '잇 있 '),
    @('Backspace by jamo', 0, 0, 'KeyG KeyK KeyS Backspace Space', '하 '),
    @('F10 → gks, F10 → GKS', 0, 0, 'KeyG KeyK KeyS F10 F10 Space', 'GKS '),
    @('Shift+Z is ㅋ', 0, 0, 'S-KeyZ KeyK Space', '카 '),
    @('F9 full width', 0, 0, 'KeyG KeyK KeyS F9 Space', 'ｇｋｓ '),
    @('F6: the staged keys as hiragana', 0, 0, 'KeyG KeyK KeyS F6 Enter', 'きのと'),
    @('F7: the staged keys as katakana', 0, 0, 'KeyG KeyK KeyS F7 Enter', 'キノト'),
    @('変換 in JIS Kana: back to Mode A, forced Korean', 0, 0, 'KeyG KeyK KeyS F6 Convert Space', '한 '),
    @('無変換 commits, ABC types English, 変換 back to 한글', 0, 0, 'KeyG KeyK KeyS NonConvert KeyG Convert KeyG KeyK Space', '한g하 '),
    @('Shift+F8 hanja, Enter picks', 0, 0, 'KeyG KeyK KeyR KeyR KeyY S-F8 Enter Space', '學校 '),
    @('Mode A slot keys: @ key → ㅡㄹ (을)', 0, 0, 'BracketLeft Space', '을 '),
    @('한/영: ABC types English, back to 한글', 0, 0, 'KeyG KeyK Hangul KeyG KeyK Space Hangul KeyG KeyK Space', '하gk 하 '),
    @('predictions: hello', 0, 1, 'KeyH KeyE KeyL KeyL KeyO Space', 'hello '),
    @('predictions: 한글 stays Korean', 0, 1, 'KeyG KeyK KeyS KeyR KeyM KeyF Space', '한글 '),
    @('predictions: Shift+Enter commits hello, no new line', 0, 1, 'KeyH KeyE KeyL KeyL KeyO S-Enter', 'hello'),
    @('predictions: ` (Backquote, the HHKB key) types `', 0, 1, 'KeyH KeyE KeyL KeyL KeyO Backquote Space', 'hello` '),
    @('predictions: Esc commits what is shown', 0, 1, 'KeyH KeyE KeyL KeyL KeyO Escape', 'hello'),
    @('predictions: Esc leaves a chosen row, Esc again commits', 0, 1, 'KeyH KeyE KeyL KeyL KeyO Tab Tab Escape Escape', 'hello'),
    @('predictions: ↓ ↓ ↑ and Tab Tab Shift+Tab move through the list', 0, 1, 'KeyH KeyE KeyL KeyL KeyO ArrowDown ArrowDown ArrowUp Space KeyH KeyE KeyL KeyL KeyO Tab Tab S-Tab Space', 'hello hello '),
    @('a retired mode (Mode B) is Mode A', 1, 0, 'KeyG KeyK KeyS Space', '한 '),
    @('JIS Kana from the menu (Mode 11)', 11, 0, 'KeyG KeyK KeyS Enter', 'きのと'),
    @('ABC in Mode A: JIS English (default)', 0, 0, 'Hangul KeyG S-Digit2 BracketLeft Hangul', 'g"@', 'unset'),
    @('ABC: JIS Dvorak', 0, 0, 'Hangul KeyS KeyD KeyF S-KeyJ Hangul', 'oeuH', 7),
    @('ABC: Windows layout passes keys', 0, 0, 'Hangul KeyG S-Digit2 Hangul', 'g@', 'windows'),
    @('ABC in Mode A, then 한글 again', 0, 0, 'Hangul KeyG KeyK Hangul KeyG KeyK Space', 'gk하 ', 'unset'),
    @('無変換 in ABC stays, 変換 from ABC to 한글', 0, 0, 'NonConvert NonConvert KeyG Convert KeyG KeyK Space', 'g하 ', 'unset')
)
# Password fields: every key passes through untouched (ABC), with predictions on.
# (The leading comma keeps a one-case list a list of cases: PowerShell would unroll it.)
$passwordCases = @(
    ,@('password field passes keys through', 0, 1, 'KeyG KeyK KeyS S-KeyR Digit1 KeyH KeyE KeyL KeyL KeyO', 'gksR1hello')
)
# Hosts that misbehave (jk_testpad's options and targets): name, Mode, Predictions, keys, expected,
# targets, extra jk_testpad arguments.
$hostCases = @(
    @('#5 auto-repeat: one jamo per repeat', 0, 0, 'D-KeyG D-KeyG D-KeyG D-KeyG R-KeyG KeyE KeyE Space', 'ㅎㅎㅎㅎㄷㄷ ', @('edit', 'rich', 'tsf'), @()),
    @('#5 a key tested twice runs once', 0, 0, 'KeyE KeyK KeyE KeyK Space', '다다 ', @('edit', 'rich', 'tsf'), @('--peek', '1', '--probe', '1')),
    @('#5 auto-repeat tested twice', 0, 0, 'D-KeyG D-KeyG D-KeyG R-KeyG Space', 'ㅎㅎㅎ ', @('tsf'), @('--peek', '1', '--probe', '1')),
    @('#4 Unicode-injected characters pass as typed', 0, 0, 'U+0061 U+0020 U+0062 U+002C U+002F U+0030 U+0035 U+0056 U+0073 U+007D U+AC10', 'a b,/05Vs}감', @('edit', 'rich', 'tsf'), @()),
    @('#4 Unicode-injected character commits first', 0, 1, 'KeyG KeyK U+0020 KeyG KeyK U+0078', '하 하x', @('edit', 'rich', 'tsf'), @()),
    @('#3 no input context: keys pass, nothing follows focus', 0, 1, 'KeyW KeyA KeyS KeyD @focus=edit KeyG KeyK KeyS Space', '한 ', @('nocontext'), @()),
    @('#6 an app ending each composition: syllables still compose', 0, 0, 'KeyE KeyK KeyF KeyK KeyQ KeyK KeyD Space', '다라방 ', @('canceltsf'), @()),
    @('#6 an app ending each composition: Backspace, Shift+F8', 0, 0, 'KeyG KeyK KeyS KeyR KeyM KeyF Backspace Backspace Space KeyG KeyK KeyR KeyR KeyY S-F8 Enter Space', '한ㄱ 學校 ', @('canceltsf'), @()),
    @('#6 an app ending each composition: predictions', 0, 1, 'KeyH KeyE KeyL KeyL KeyO Tab Tab Space KeyG KeyK KeyS ArrowLeft KeyK Space', 'Hello 아 한', @('canceltsf'), @()),
    @('#9 a virtual key without a scan code types its letter', 0, 0, 'VK:47 VK:4B VK:53 Space', '한 ', @('edit', 'rich', 'tsf'), @()),
    @('#4 Unicode-injected characters in ABC', 0, 0, 'Hangul U+0061 U+0020 U+0030 Hangul', 'a 0', @('edit', 'tsf'), @())
)

$saved = @{}
foreach ($name in 'Mode', 'EnglishMode', 'LearnFromChoices', 'PredictiveAccepted') {
    $saved[$name] = (Get-ItemProperty $key -Name $name -ErrorAction SilentlyContinue).$name
}
$savedApp = (Get-ItemProperty $appKey -Name Predictions -ErrorAction SilentlyContinue).Predictions
Set-ItemProperty $key LearnFromChoices 0 -Type DWord
Set-ItemProperty $key PredictiveAccepted 1 -Type DWord
$failed = 0
$out = Join-Path $env:TEMP 'jk-type-test.txt'
try {
    foreach ($c in @($cases) + @($passwordCases) + @($hostCases)) {
        if ($Filter -and $c[0] -notlike "*$Filter*") { continue }
        $hosted = $hostCases -contains $c
        Set-ItemProperty $key Mode $c[1] -Type DWord
        Set-ItemProperty $appKey Predictions $c[2] -Type DWord
        $english = if ($c.Count -gt 5 -and -not $hosted) { $c[5] } else { 'unset' }
        if ($english -eq 'unset') { Remove-ItemProperty $key -Name EnglishMode -ErrorAction SilentlyContinue }
        elseif ($english -eq 'windows') { Set-ItemProperty $key EnglishMode ([int32]-1) -Type DWord }
        else { Set-ItemProperty $key EnglishMode $english -Type DWord }
        $targets = if ([object]::ReferenceEquals($c, $passwordCases[0])) { @('password', 'pwscope') }
            elseif ($hosted) { $c[5] } else { $Target }
        $extra = if ($hosted) { $c[6] } else { @() }
        foreach ($t in $targets) {
            Remove-Item $out -ErrorAction SilentlyContinue
            $p = Start-Process $pad -ArgumentList (@('--profile', 'on', '--target', $t, '--keys', "`"$($c[3])`"", '--out', "`"$out`"") + $extra) -PassThru
            if (-not $p.WaitForExit(30000)) { $p.Kill(); Write-Host "TIMEOUT $($c[0]) [$t]" -ForegroundColor Red; $failed++; continue }
            $got = if (Test-Path $out) { [IO.File]::ReadAllText($out, [Text.Encoding]::UTF8) } else { '<no output>' }
            if ($got -ceq $c[4]) {
                Write-Host "ok   $($c[0]) [$t]"
            } else {
                Write-Host "FAIL $($c[0]) [$t]: got [$got], expected [$($c[4])]" -ForegroundColor Red
                $failed++
            }
        }
    }
} finally {
    foreach ($name in $saved.Keys) {
        if ($null -eq $saved[$name]) { Remove-ItemProperty $key -Name $name -ErrorAction SilentlyContinue }
        else { Set-ItemProperty $key $name $saved[$name] -Type DWord }
    }
    if ($null -eq $savedApp) { Remove-Item $appKey -ErrorAction SilentlyContinue }
    else { Set-ItemProperty $appKey Predictions $savedApp -Type DWord }
}
if ($failed) { Write-Host "$failed failed" -ForegroundColor Red; exit 1 }
Write-Host 'all passed'
