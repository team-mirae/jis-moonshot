; The JIS Moonshot installer (Inno Setup 6). Build it with tools/package.ps1, which passes:
;   Version   the engine's Cargo.toml version, e.g. 0.3.1
;   Build     windows/build, holding x64 and x86
;   Sign      defined when the installer and its uninstaller are signed (SignTool "jk")
; It does what tools/install.ps1 does, plus the predictive-typing notice (docs/moonshot.md §6),
; which must be accepted before installing. It installs beside JIS Korean: its own AppId, CLSID,
; profile, folders and registry key.

#ifndef Version
  #error Build with tools/package.ps1
#endif
#define Root "..\.."
#define Clsid "{EC381A55-0AD1-42DF-BBAB-AF4F34F90C1C}"
; Shown in Apps & features and on the finish page.
#define Publisher "Mirae Research"
#define DocsUrl "https://jis-korean.sjun.me"

[Setup]
; Permanent: Windows finds an earlier install (to upgrade it) by this id. Never change it.
AppId={{D7B8094A-C54B-442D-BF74-A8E44BF036C7}
AppName=JIS Moonshot
AppVersion={#Version}
AppVerName=JIS Moonshot {#Version}
AppPublisher={#Publisher}
AppPublisherURL={#DocsUrl}
AppSupportURL={#DocsUrl}
VersionInfoVersion={#Version}
AppCopyright=Copyright 2026 Mirae Research & Min Jun Sung
; tools/install.ps1's folder, so an upgrade over a development install replaces it in place.
DefaultDirName={autopf}\JIS Moonshot
DisableDirPage=yes
DisableProgramGroupPage=yes
DisableReadyPage=yes
DisableWelcomePage=no
LicenseFile={#Root}\LICENSE
PrivilegesRequired=admin
; Windows 11 is 64-bit only; the x86 DLL is for 32-bit apps on it.
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
; Every app has the DLL loaded: never offer to close them. The old DLL is moved aside instead
; (RenameAside below), and apps load the new one when they restart.
CloseApplications=no
RestartApplications=no
SetupIconFile={#Root}\windows\icons\moonshot.ico
UninstallDisplayIcon={app}\moonshot.ico
UninstallDisplayName=JIS Moonshot
WizardStyle=modern
Compression=lzma2/max
SolidCompression=yes
OutputDir={#Build}\dist
OutputBaseFilename=JISMoonshot-{#Version}-setup
#ifdef Sign
SignTool=jk
SignedUninstaller=yes
#endif

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"

[Messages]
; Not the default "close all other applications": apps can stay open, and restart afterwards.
WelcomeLabel2=This will install [name/ver] on your computer.%n%nYou don't need to close other apps. Apps that are open keep the old version, or none, until they restart.
ConfirmUninstall=Remove JIS Moonshot from this PC and from your keyboards?
FinishedHeadingLabel=JIS Moonshot is installed
FinishedLabelNoIcons=Restart the apps that were open: they keep the old version, or none, until they restart.%n%nPress Win+Space and pick JIS Moonshot.%n%n無変換 switches to English, 変換 back to Korean (한/영 toggles them). F6 types hiragana, F7 katakana. Right-click the 한글 / ABC indicator in the taskbar for JIS Kana and the settings.
FinishedLabel=Restart the apps that were open: they keep the old version, or none, until they restart.%n%nPress Win+Space and pick JIS Moonshot.%n%n無変換 switches to English, 変換 back to Korean (한/영 toggles them). F6 types hiragana, F7 katakana. Right-click the 한글 / ABC indicator in the taskbar for JIS Kana and the settings.

[Files]
; The engine's data (the hanja dictionary, the predictor's lexicons, predict/common-ja.txt for
; the kana reading included) is embedded in the DLL (jismoonshot.rc.in).
Source: "{#Build}\x64\jismoonshot.dll"; DestDir: "{app}\x64"; Flags: ignoreversion
Source: "{#Build}\x86\jismoonshot.dll"; DestDir: "{app}\x86"; Flags: ignoreversion
; The licenses of the data inside the DLL.
Source: "{#Root}\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Root}\engine\dict\LICENSE-libhangul-hanja.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Root}\engine\dict\LICENSE-unicode.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Root}\engine\predict\LICENSE-corpora.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Root}\engine\predict\LICENSE-wordnet.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Root}\windows\icons\moonshot.ico"; DestDir: "{app}"; Flags: ignoreversion
; Run by Setup and the uninstaller as the user, to change their language list.
Source: "keyboards.ps1"; DestDir: "{app}"; Flags: ignoreversion

[Run]
Filename: "{#DocsUrl}"; Description: "Open the JIS Moonshot docs"; Flags: postinstall shellexec nowait skipifsilent unchecked runasoriginaluser

[Code]
const
  MOVEFILE_DELAY_UNTIL_REBOOT = 4;

var
  ConsentPage: TOutputMsgMemoWizardPage;
  AcceptButton: TNewRadioButton;
  DeclineButton: TNewRadioButton;
  RemoveUserData: Boolean;
  LeftForRestart: Boolean;

{ MoveFileExW with no new name: delete the file when Windows restarts. }
function DeleteAtRestart(Existing: String; NoNewName, Flags: Cardinal): Boolean;
  external 'MoveFileExW@kernel32.dll stdcall';

function PowerShell(Params: String): String;
begin
  Result := '-NoProfile -NonInteractive -ExecutionPolicy Bypass -WindowStyle Hidden -File "'
    + ExpandConstant('{app}\keyboards.ps1') + '" ' + Params;
end;

{ Runs keyboards.ps1 as the user who started Setup: the language list is per user, and the
  elevated process may be another account. The uninstaller can't do that (Inno Setup has no
  ExecAsOriginalUser there): it changes the list of the account it was elevated as, the same
  user unless they elevated with another account's password. }
function RunKeyboards(Params: String): Boolean;
var
  Exe: String;
  Code: Integer;
begin
  Exe := ExpandConstant('{sys}\WindowsPowerShell\v1.0\powershell.exe');
  if IsUninstaller then
    Result := Exec(Exe, PowerShell(Params), '', SW_HIDE, ewWaitUntilTerminated, Code)
  else
    Result := ExecAsOriginalUser(Exe, PowerShell(Params), '', SW_HIDE, ewWaitUntilTerminated, Code);
  Result := Result and (Code = 0);
end;

function RegSvr(Arch, Params: String): Boolean;
var
  Exe: String;
  Code: Integer;
begin
  if Arch = 'x64' then Exe := ExpandConstant('{sys}\regsvr32.exe')
  else Exe := ExpandConstant('{syswow64}\regsvr32.exe');
  Result := Exec(Exe, Params + ' "' + ExpandConstant('{app}\' + Arch + '\jismoonshot.dll') + '"',
    '', SW_HIDE, ewWaitUntilTerminated, Code) and (Code = 0);
end;

{ Deletes the copies earlier installs moved aside, where no process holds them any more. The
  others stay, or with AtRestart are deleted when Windows restarts; returns whether any did. }
function DeleteOldCopies(Dir: String; AtRestart: Boolean): Boolean;
var
  Find: TFindRec;
begin
  Result := False;
  if FindFirst(Dir + '\jismoonshot.dll.old-*', Find) then begin
    try
      repeat
        if not DeleteFile(Dir + '\' + Find.Name) and AtRestart then begin
          DeleteAtRestart(Dir + '\' + Find.Name, 0, MOVEFILE_DELAY_UNTIL_REBOOT);
          Result := True;
        end;
      until not FindNext(Find);
    finally
      FindClose(Find);
    end;
  end;
end;

{ A process with the TIP loaded keeps the file open, but it can be renamed: move it aside so new
  processes load the new build. Returns the new name, or '' if the file is gone. }
function RenameAside(Dir: String): String;
var
  Target: String;
begin
  Result := '';
  Target := Dir + '\jismoonshot.dll';
  if FileExists(Target) and not DeleteFile(Target) then begin
    Result := Target + '.old-' + GetDateTimeString('yyyymmddhhnnsszzz', #0, #0);
    if not RenameFile(Target, Result) then
      RaiseException('Can''t replace ' + Target + '.');
  end;
end;

{ The predictive-typing notice (docs/moonshot.md §6), as the TIP shows it (text_service.cpp
  kConsentNotice) and macOS (mac/Sources/Consent.swift). Keep the three in step. }
function ConsentNotice: String;
begin
  Result :=
    'JIS Moonshot reads every key you type as Korean, English and Japanese kana at once, and ' +
    'ranks those readings to show the likeliest one. That is predictive typing, and JIS ' +
    'Moonshot doesn''t work without it.' + #13#10#13#10 +
    '• What it reads: the keys of the text you haven''t committed yet, to rank its readings.' + #13#10 +
    '• Where: only on this PC. Nothing is sent anywhere.' + #13#10 +
    '• Learning from your choices is separate and off until you turn it on in the indicator ' +
    'menu.' + #13#10 +
    '• Terminals, password managers and similar programs start with predictions off ' +
    '(Predictions in <program> in the indicator menu). There, keys type forced Korean.' + #13#10#13#10 +
    'Until you accept, JIS Moonshot passes every key to the app untouched.';
end;

{ Accepted on the notice page; in a silent install (no pages), only with /ACCEPTPREDICTIVE=yes.
  Otherwise JIS Moonshot is installed but passes every key through until it's accepted from the
  indicator's menu. }
function PredictiveAccepted: Boolean;
begin
  if WizardSilent then
    Result := Lowercase(ExpandConstant('{param:acceptpredictive|no}')) = 'yes'
  else
    Result := AcceptButton.Checked;
end;

procedure UpdateConsentNext(Sender: TObject);
begin
  if WizardForm.CurPageID = ConsentPage.ID then
    WizardForm.NextButton.Enabled := AcceptButton.Checked;
end;

procedure InitializeWizard;
begin
  { After the licence, a page like it: the notice, and Next only once it's accepted. }
  ConsentPage := CreateOutputMsgMemoPage(wpLicense, 'Predictive Typing',
    'Please read the following before installing JIS Moonshot.',
    'JIS Moonshot doesn''t work without predictive typing. Accept it to install.',
    ConsentNotice);
  ConsentPage.RichEditViewer.Height := ConsentPage.SurfaceHeight - ScaleY(48);
  AcceptButton := TNewRadioButton.Create(ConsentPage);
  AcceptButton.Parent := ConsentPage.Surface;
  AcceptButton.Caption := 'I accept predictive typing';
  AcceptButton.Left := 0;
  AcceptButton.Top := ConsentPage.RichEditViewer.Top + ConsentPage.RichEditViewer.Height + ScaleY(8);
  AcceptButton.Width := ConsentPage.SurfaceWidth;
  AcceptButton.Height := ScaleY(17);
  AcceptButton.OnClick := @UpdateConsentNext;
  DeclineButton := TNewRadioButton.Create(ConsentPage);
  DeclineButton.Parent := ConsentPage.Surface;
  DeclineButton.Caption := 'I do not accept';
  DeclineButton.Left := 0;
  DeclineButton.Top := AcceptButton.Top + AcceptButton.Height + ScaleY(4);
  DeclineButton.Width := ConsentPage.SurfaceWidth;
  DeclineButton.Height := ScaleY(17);
  DeclineButton.Checked := True;
  DeclineButton.OnClick := @UpdateConsentNext;
end;

procedure CurPageChanged(CurPageID: Integer);
begin
  if CurPageID = ConsentPage.ID then UpdateConsentNext(nil);
end;

function NextButtonClick(CurPageID: Integer): Boolean;
begin
  { The Next button is disabled until it's accepted; this holds if something enables it anyway. }
  Result := (CurPageID <> ConsentPage.ID) or AcceptButton.Checked;
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  Arch, Data, Params: String;
  I, Code: Integer;
begin
  if CurStep = ssInstall then begin
    for I := 0 to 1 do begin
      if I = 0 then Arch := 'x64' else Arch := 'x86';
      DeleteOldCopies(ExpandConstant('{app}\' + Arch), False);
      RenameAside(ExpandConstant('{app}\' + Arch));
    end;
  end else if CurStep = ssPostInstall then begin
    { The log folder, writable by every process, AppContainer (Store) apps included. }
    Data := ExpandConstant('{commonappdata}\JIS Moonshot');
    ForceDirectories(Data);
    Exec(ExpandConstant('{sys}\icacls.exe'),
      '"' + Data + '" /grant *S-1-15-2-1:(OI)(CI)M *S-1-5-32-545:(OI)(CI)M',
      '', SW_HIDE, ewWaitUntilTerminated, Code);
    if not RegSvr('x64', '/s') or not RegSvr('x86', '/s') then
      MsgBox('JIS Moonshot couldn''t be registered with Windows. Run Setup again; if it keeps ' +
        'failing, see ' + '{#DocsUrl}.', mbError, MB_OK);
    { As the user who started Setup: their keyboards, and PredictiveAccepted in their HKCU. }
    Params := '';
    if PredictiveAccepted then Params := '-AcceptPredictive';
    if not RunKeyboards(Params) then
      MsgBox('JIS Moonshot is installed, but it couldn''t be added to your keyboards. Add it in ' +
        'Settings > Time & language > Language & region > Korean > Language options.',
        mbInformation, MB_OK);
  end;
end;

{ After the uninstaller's own "remove JIS Moonshot?" (ConfirmUninstall): whether to delete the user's own
  files too. Unticked by default. }
procedure AskAboutUserData;
var
  Form: TSetupForm;
  Text: TNewStaticText;
  Box: TNewCheckBox;
  Ok: TNewButton;
begin
  Form := CreateCustomForm(ScaleX(400), ScaleY(150), False, True);
  try
    Form.Caption := 'Uninstall JIS Moonshot';
    Text := TNewStaticText.Create(Form);
    Text.Parent := Form;
    Text.Left := ScaleX(16);
    Text.Top := ScaleY(16);
    Text.Width := Form.ClientWidth - ScaleX(32);
    Text.WordWrap := True;
    Text.Caption := 'Your training file and learned weights stay on this PC, for when you ' +
      'install JIS Moonshot again.';
    Box := TNewCheckBox.Create(Form);
    Box.Parent := Form;
    Box.Left := ScaleX(16);
    Box.Top := Text.Top + Text.Height + ScaleY(16);
    Box.Width := Form.ClientWidth - ScaleX(32);
    Box.Height := ScaleY(34);
    Box.Caption := 'Delete them too (%APPDATA%\JIS Moonshot)';
    Ok := TNewButton.Create(Form);
    Ok.Parent := Form;
    Ok.Caption := 'Continue';
    Ok.ModalResult := mrOk;
    Ok.Default := True;
    Ok.Cancel := True;
    Ok.Width := ScaleX(80);
    Ok.Height := ScaleY(24);
    Ok.Top := Form.ClientHeight - Ok.Height - ScaleY(12);
    Ok.Left := Form.ClientWidth - Ok.Width - ScaleX(16);
    Form.ShowModal;
    RemoveUserData := Box.Checked;
  finally
    Form.Free;
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  Arch, Dir, Aside: String;
  I: Integer;
begin
  if CurUninstallStep = usUninstall then begin
    if not UninstallSilent then AskAboutUserData;
    RunKeyboards('-Remove');
    for I := 0 to 1 do begin
      if I = 0 then Arch := 'x64' else Arch := 'x86';
      if FileExists(ExpandConstant('{app}\' + Arch + '\jismoonshot.dll')) then RegSvr(Arch, '/s /u');
    end;
    RegDeleteKeyIncludingSubkeys(HKLM64, 'SOFTWARE\Microsoft\CTF\TIP\{#Clsid}');
    RegDeleteKeyIncludingSubkeys(HKLM64, 'SOFTWARE\Classes\CLSID\{#Clsid}');
    RegDeleteKeyIncludingSubkeys(HKLM32, 'SOFTWARE\Classes\CLSID\{#Clsid}');
    { Processes that loaded the DLL keep it open until they exit: move it aside and delete it
      when Windows restarts. }
    for I := 0 to 1 do begin
      if I = 0 then Arch := 'x64' else Arch := 'x86';
      Dir := ExpandConstant('{app}\' + Arch);
      if DeleteOldCopies(Dir, True) then LeftForRestart := True;
      Aside := RenameAside(Dir);
      if Aside <> '' then begin
        DeleteAtRestart(Aside, 0, MOVEFILE_DELAY_UNTIL_REBOOT);
        LeftForRestart := True;
      end;
      if LeftForRestart then DeleteAtRestart(Dir, 0, MOVEFILE_DELAY_UNTIL_REBOOT);
    end;
  end else if CurUninstallStep = usPostUninstall then begin
    if LeftForRestart then
      DeleteAtRestart(ExpandConstant('{app}'), 0, MOVEFILE_DELAY_UNTIL_REBOOT);
    if RemoveUserData then
      DelTree(ExpandConstant('{userappdata}\JIS Moonshot'), True, True, True);
    if LeftForRestart and not UninstallSilent then
      MsgBox('Some apps still have JIS Moonshot loaded. Its last files are deleted when Windows ' +
        'restarts.', mbInformation, MB_OK);
  end;
end;
