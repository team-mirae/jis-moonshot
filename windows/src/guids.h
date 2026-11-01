// JIS Moonshot identifiers. These GUIDs are permanent: they end up in users' registries and
// language lists (InputMethodTips "0412:{CLSID}{PROFILE}"). Never change them. They are JIS
// Moonshot's own, not JIS Korean's, so the two install side by side (docs/moonshot.md §7);
// kLangBarItemInputMode and kPropInputScope are Windows' and stay as they are.
#pragma once

#include <guiddef.h>

namespace jk {

// The TIP's COM class. {EC381A55-0AD1-42DF-BBAB-AF4F34F90C1C}
inline constexpr GUID kClsidTextService = {
    0xec381a55, 0x0ad1, 0x42df, {0xbb, 0xab, 0xaf, 0x4f, 0x34, 0xf9, 0x0c, 0x1c}};

// The one language profile, ko-KR. The mode (spec §3.6) is a setting in its menu, not a profile
// per mode as on macOS (spec §8.4). {42BB462F-CA66-4FD0-9289-951EAE5C4BB3}
inline constexpr GUID kProfile = {
    0x42bb462f, 0xca66, 0x4fd0, {0x92, 0x89, 0x95, 0x1e, 0xae, 0x5c, 0x4b, 0xb3}};

// The input indicator's mode button (Windows 8+). The SDK headers don't declare it; the value is
// Windows' own (as in Microsoft's SampleIME). {2C77A81E-41CC-4178-A3A7-5F8A987568E6}
inline constexpr GUID kLangBarItemInputMode = {
    0x2c77a81e, 0x41cc, 0x4178, {0xa3, 0xa7, 0x5f, 0x8a, 0x98, 0x75, 0x68, 0xe6}};

// Display attribute of unconfirmed text (underline). {DAE8D0C3-482D-4624-976F-06C5BCD6C882}
inline constexpr GUID kDisplayAttributeInput = {
    0xdae8d0c3, 0x482d, 0x4624, {0x97, 0x6f, 0x06, 0xc5, 0xbc, 0xd6, 0xc8, 0x82}};

// Display attribute of a reading shown in place of the typed one: a prediction (thick
// underline). {11816B59-0283-4416-975C-78128F1D21DC}
inline constexpr GUID kDisplayAttributeConverted = {
    0x11816b59, 0x0283, 0x4416, {0x97, 0x5c, 0x78, 0x12, 0x8f, 0x1d, 0x21, 0xdc}};

// Global compartment (shared by every app in the session, never stored): nonzero while 한/영 has
// switched JIS Moonshot to English. {0F89DFD1-CCEB-4DB1-B2E0-1DC728F09D2B}
inline constexpr GUID kCompartmentEnglish = {
    0x0f89dfd1, 0xcceb, 0x4db1, {0xb2, 0xe0, 0x1d, 0xc7, 0x28, 0xf0, 0x9d, 0x2b}};

// The text property holding a field's input scopes (InputScope.h's GUID_PROP_INPUTSCOPE, which
// no import library defines). {1713DD5A-68E7-4A5B-9AF6-592A595C778D}
inline constexpr GUID kPropInputScope = {
    0x1713dd5a, 0x68e7, 0x4a5b, {0x9a, 0xf6, 0x59, 0x2a, 0x59, 0x5c, 0x77, 0x8d}};

inline constexpr unsigned short kLangIdKorean = 0x0412;

}  // namespace jk
