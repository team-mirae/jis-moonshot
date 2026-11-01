// User settings: HKCU\Software\JIS Moonshot, else HKLM\Software\JIS Moonshot, else the default.
// HKLM is the fallback for apps that don't see the user's key (FKeyIME measured it in packaged
// apps). Values:
//
//   PredictiveAccepted DWORD  0  the version of the predictive-typing notice accepted (docs/
//                                moonshot.md §6); below kConsentVersion every key passes through
//   Mode               DWORD  0  the mode by engine id: 0 A (JIS Moonshot), 11 A_KANA (JIS Kana).
//                                An id the engine refuses (the retired 1, 2, 9, 10) is Mode A
//   EnglishMode        DWORD  3  ABC's (無変換, 한/영) layout: 3 A_QWERTY, 7 A_DVORAK, 8 A_COLEMAK,
//                                or 0xFFFFFFFF for Windows' own
//   EnterCommits       DWORD  0  Enter commits without a new line (spec §5.1)
//   LearnFromChoices   DWORD  0  learning from the user's choices (spec §10.11)
//   Learned            SZ        what was learned: one line of counts, no text or keys
//   Weight.<Scenario>  SZ        user tuning (spec §10.5): Korean English Implied Mixed TwoSet
//                                KeepFinals Hanja ImpliedL Kana, e.g. "0.8" (Hanja: retired, no effect)
//   DiagnosticLog      DWORD  0  log.h
//   SettingsVersion    DWORD     changes with every write below (Version)
//   Apps\<exe>         key       per app: Predictions DWORD, overriding the default-off list;
//                                Off DWORD 1: JIS Moonshot types ABC's layout there
//
// Predictions are always on (docs/moonshot.md §6): there is no global switch, only the per-app
// exclusions, which are a safety measure.
#pragma once

#include <optional>
#include <string>

namespace jk::settings {

// Changes with every write through this file (SettingsVersion, HKCU): engines re-read the settings
// only when it differs from the one they applied. Values changed by hand in the registry apply in
// programs started afterwards, or with the next change made from the menu.
unsigned long Version();

// The version of the predictive-typing notice (docs/moonshot.md §6). Bump it when the notice
// changes in substance: everyone is asked again.
inline constexpr unsigned long kConsentVersion = 1;
// Whether the notice was accepted (PredictiveAccepted >= kConsentVersion), and accepting it.
bool ConsentAccepted();
void AcceptConsent();

unsigned long ReadDword(const wchar_t* name, unsigned long fallback);
void WriteDword(const wchar_t* name, unsigned long value);
std::optional<std::wstring> ReadString(const wchar_t* name);
void WriteString(const wchar_t* name, const std::wstring& value);
void Delete(const wchar_t* name);

// The running program's file name, lowercase ("notepad.exe").
const std::wstring& ExeName();

// Why predictions start off in `exe` (spec §10.11.1: terminals, remote desktop, password
// managers...), or nullptr when they start on.
const wchar_t* DefaultOffReason(const std::wstring& exe);
// Whether predictions are on in `exe`: the user's choice for it, else the default-off list.
bool AppPredicting(const std::wstring& exe);
void SetAppPredicting(const std::wstring& exe, bool on);

// Whether JIS Moonshot is off in `exe`: every key types ABC's layout, there only. For
// programs whose password boxes don't say they are (banking and certificate programs).
bool AppOff(const std::wstring& exe);
void SetAppOff(const std::wstring& exe, bool off);

}  // namespace jk::settings
