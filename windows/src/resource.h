#pragma once

// The logo: the first icon (index 0), which the profile names.
#define IDI_LOGO 101

// The profile's display name (Windows Settings, the input indicator).
#define IDS_PROFILE 201

// Engine data, embedded so every process maps one copy from the DLL image.
// 301 was the hanja dictionary (gone).
#define IDR_RANKER 302
#define IDR_COMMON_EN 303
#define IDR_COMMON_KO 304
#define IDR_NOUNS 305
#define IDR_LEARNED 306
#define IDR_COMMON_JA 307  // the kana reading's lexicon (docs/moonshot.md §5)
#define IDR_KANJI 308      // kana-kanji conversion (docs/moonshot.md §5.1)
