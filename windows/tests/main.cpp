// Shell tests: the scan code map (spec §2), and key sequences typed through it into the engine,
// as the text service does. Behavior itself is the engine's (engine/tests); this checks the wiring.
#include <cmath>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>

#include "jis_korean.h"
#include "json.h"
#include "key_map.h"
#include "log.h"
#include "training_lines.h"

namespace {

int failures = 0;

void Check(bool ok, const char* what) {
    if (!ok) {
        std::printf("FAIL %s\n", what);
        ++failures;
    }
}

std::string Str(const std::uint8_t* p, size_t n) { return std::string(reinterpret_cast<const char*>(p), n); }

struct Press {
    unsigned vk;
    unsigned scan;
    bool shift = false;
};

// Types `presses` into a fresh engine in `mode`; returns everything committed + the preedit.
std::string Type(unsigned mode, const std::vector<Press>& presses, std::string* preedit = nullptr) {
    JkEngine* e = jk_new();
    jk_set_mode(e, mode);
    std::string committed;
    for (const Press& p : presses) {
        jk_key(e, jk::EngineKey(p.vk, p.scan, false), p.shift ? JK_MOD_SHIFT : 0);
        committed += Str(jk_commit_ptr(e), jk_commit_len(e));
    }
    if (preedit) *preedit = Str(jk_preedit_ptr(e), jk_preedit_len(e));
    jk_free(e);
    return committed;
}

size_t Format(wchar_t* line, size_t size, size_t used, const wchar_t* format, ...) {
    va_list args;
    va_start(args, format);
    const size_t n = jk::log::Append(line, size, used, format, args);
    va_end(args);
    return n;
}

// The key code name of an engine key, as the engine spells it.
std::string Code(std::uint32_t key) { return Str(jk_key_code_ptr(key), jk_key_code_len(key)); }

}  // namespace

int main() {
    // Spec §2: scan codes → KeyboardEvent.code names, whatever the virtual key.
    struct Case {
        unsigned scan;
        const char* code;
    };
    const Case cases[] = {
        {0x02, "Digit1"}, {0x0B, "Digit0"}, {0x0C, "Minus"},       {0x0D, "Equal"},
        {0x7D, "IntlYen"}, {0x10, "KeyQ"},  {0x19, "KeyP"},        {0x1A, "BracketLeft"},
        {0x1B, "BracketRight"}, {0x1E, "KeyA"}, {0x26, "KeyL"},    {0x27, "Semicolon"},
        {0x28, "Quote"},  {0x2B, "Backslash"}, {0x2C, "KeyZ"},     {0x32, "KeyM"},
        {0x33, "Comma"},  {0x34, "Period"}, {0x35, "Slash"},       {0x73, "IntlRo"},
        {0x29, "Backquote"}, {0x56, "IntlBackslash"},
    };
    for (const Case& c : cases) {
        const std::uint32_t key = jk::EngineKey(0xE5 /* VK_PROCESSKEY: ignored */, c.scan, false);
        Check(key != JK_KEY_OTHER && Code(key) == c.code, c.code);
    }
    Check(jk::EngineKey(0x6F, 0x35, true) == JK_KEY_OTHER, "keypad / is not Slash");
    Check(jk::EngineKey(0x0D, 0x1C, true) == JK_KEY_ENTER, "keypad Enter is Enter");
    Check(jk::EngineKey(0x10, 0x2A, false) == JK_KEY_MODIFIER, "Shift alone");
    Check(jk::EngineKey(0x14, 0x3A, false) == JK_KEY_MODIFIER, "Caps Lock alone");
    Check(jk::EngineKey(0x75, 0x40, false) == JK_KEY_F6, "F6");
    Check(jk::EngineKey(0x79, 0x44, false) == JK_KEY_F10, "F10");
    Check(jk::EngineKey(0x24, 0x47, true) == JK_KEY_OTHER, "Home");
    Check(jk::EngineKey(0x15, 0x38, true) == JK_KEY_OTHER, "한/영 isn't an engine key");
    // 한/영, 無変換 and 変換 switch (docs/moonshot.md §4, §7): by virtual key, or 無変換 / 変換 by
    // their JIS scan codes; never engine keys.
    Check(jk::SwitchKeyOf(0x15, 0x01380001) == jk::SwitchKey::kHangul, "VK_HANGUL is 한/영");
    Check(jk::SwitchKeyOf(0x1D, 0x007B0001) == jk::SwitchKey::kNonConvert, "VK_NONCONVERT is 無変換");
    Check(jk::SwitchKeyOf(0x1C, 0x00790001) == jk::SwitchKey::kConvert, "VK_CONVERT is 変換");
    Check(jk::SwitchKeyOf(0xFF, 0x007B0001) == jk::SwitchKey::kNonConvert, "無変換 by its scan code");
    Check(jk::SwitchKeyOf(0xFF, 0x00790001) == jk::SwitchKey::kConvert, "変換 by its scan code");
    Check(jk::SwitchKeyOf(0xE7, 0x00790001) == jk::SwitchKey::kNone, "VK_PACKET never switches");
    Check(jk::SwitchKeyOf('G', 0x00220001) == jk::SwitchKey::kNone, "a letter doesn't switch");
    Check(jk::EngineKey(0x1C, 0x79, false) == JK_KEY_OTHER && jk::EngineKey(0x1D, 0x7B, false) == JK_KEY_OTHER,
          "無変換 and 変換 aren't engine keys");
    // #4: VK_PACKET carries a character's low byte where the scan code goes: never a position.
    for (unsigned scan : {0x20u, 0x2Cu, 0x30u, 0x10u, 0x56u, 0x73u, 0x7Du, 0x00u}) {
        Check(jk::EngineKey(0xE7, scan, false) == JK_KEY_OTHER, "VK_PACKET is never a letter key");
    }

    // #9: a key injected with a virtual key but scan code 0 gets its scan code from the layout.
    {
        auto scan_of = [](unsigned vk) -> unsigned { return vk == 'G' ? 0x22u : vk == 0xE7 ? 0x20u : 0u; };
        Check(jk::KeyOf('G', 0x00000001, scan_of) == JK_KEY_KEYG, "scan 0 + VK G → KeyG");
        Check(jk::KeyOf('G', 0x00220001, scan_of) == JK_KEY_KEYG, "a real scan code is kept");
        Check(jk::KeyOf('K', 0x00220001, scan_of) == JK_KEY_KEYG, "the scan code wins over the VK");
        Check(jk::KeyOf(0xE7, 0x00000001, scan_of) == JK_KEY_OTHER, "VK_PACKET doesn't take the scan-0 path");
        Check(jk::KeyOf(0x25, 0x014B0001, scan_of) == JK_KEY_ARROWLEFT, "extended arrows by VK");
        Check(jk::KeyOf('G', 0x00000001, nullptr) == JK_KEY_OTHER, "no lookup: unknown");
    }

    // #9: training file counts are clamped before the cast (1e300, -1e300, infinity).
    {
        Check(jk::training::ClampCount(1e300, 1, 1000000, 1000) == 1000000, "1e300 → the top");
        Check(jk::training::ClampCount(-1e300, 1, 1000000, 1000) == 1, "-1e300 → the bottom");
        Check(jk::training::ClampCount(std::nan(""), 1, 20, 1) == 1, "NaN → the default");
        Check(jk::training::ClampCount(HUGE_VAL, 1, 20, 1) == 1, "infinity → the default");
        Check(jk::training::ClampCount(7.9, 1, 20, 1) == 7, "in range");
        const auto list = jk::json::Parse(
            R"([{"word": "가나", "count": 1e300}, {"word": "다라", "count": -5e20}, {"word": "마바", "count": 1e999},)"
            R"( {"word": "abc", "count": 12}, {"word": "사아", "confidence": "likely"}, {"word": "자차"}])");
        Check(list.has_value(), "the vocabulary list parses");
        if (list) {
            Check(jk::training::Lines(&*list, jk::training::KoreanWord) ==
                      L"가나\t1000000\n다라\t1\n마바\t1000\n사아\t20\n자차\t1000",
                  "Lines clamps counts and skips words in the wrong script");
            Check(jk::training::Lines(&*list, jk::training::EnglishWord) == L"abc\t12", "English Lines");
        }
        // Kana words (vocabulary.japanese, avoid "japanese"): hiragana, katakana and ー only.
        const auto kana = jk::json::Parse(
            R"([{"word": "きって", "count": 5}, {"word": "コーヒー"}, {"word": "漢字"}, {"word": "abc"}, {"word": "カナa"}])");
        Check(kana && jk::training::Lines(&*kana, jk::training::KanaWord) == L"きって\t5\nコーヒー\t1000",
              "Kana Lines");
        const auto avoid = jk::json::Parse(
            R"([{"word": "きのと", "language": "japanese", "strength": "strong"}, {"word": "한", "language": "korean"},)"
            R"( {"word": "キノト", "language": "japanese"}])");
        Check(avoid && jk::training::AvoidLines(&*avoid, "japanese", jk::training::KanaWord) == L"きのと\tstrong\nキノト",
              "Japanese AvoidLines");
    }

    // #5: a key-down tested twice is recognised; separate presses and auto-repeats aren't.
    {
        const std::intptr_t press = 0x00220001, repeat = 0x40220001;  // KeyG: scan 0x22, bit 30 for repeats
        Check(jk::SameKeyDown(0x47, press, 100, 1000, 0x47, press, 100, 1000), "the same test twice");
        Check(jk::SameKeyDown(0x47, press, 90, 1000, 0x47, press, 100, 1003),
              "tested before the message was taken (previous message's time), then again");
        Check(!jk::SameKeyDown(0x47, press, 90, 1000, 0x47, press, 2000, 2000),
              "a later press after a missed key-up isn't swallowed");
        Check(!jk::SameKeyDown(0x47, repeat, 100, 1000, 0x47, repeat, 133, 1033), "auto-repeats each count");
        Check(jk::SameKeyDown(0x47, repeat, 133, 1033, 0x47, repeat, 133, 1035), "an auto-repeat tested twice");
        Check(!jk::SameKeyDown(0x47, press, 100, 1000, 0x4B, 0x00250001, 100, 1000), "another key");
    }

    // Typed through the map: G K S → 한 (unconfirmed), Space commits it with a space.
    const Press g = {0x47, 0x22}, k = {0x4B, 0x25}, s = {0x53, 0x1F}, space = {0x20, 0x39};
    std::string preedit;
    Check(Type(0, {g, k, s}, &preedit).empty() && preedit == "한", "G K S → 한");
    Check(Type(0, {g, k, s, space}) == "한 ", "한 Space");
    // Spec §6.2: F10 → gks.
    Check((Type(0, {g, k, s, {0x79, 0x44}}, &preedit), preedit == "gks"), "한 F10 → gks");
    // Shift+Z is ㅋ, no longer F10 (docs/moonshot.md §1).
    Check((Type(0, {{0x5A, 0x2C, true}, k}, &preedit), preedit == "카"), "Shift+Z K → 카");
    // Number row: 1 → ㅃ.
    Check((Type(0, {{0x31, 0x02}}, &preedit), preedit == "ㅃ"), "1 → ㅃ");
    // Mode A: the @ key (US [) is ' (spec §3.2).
    Check((Type(0, {{0xDB, 0x1A}}, &preedit), preedit == "'"), "Mode A @ → '");
    // The modes (docs/moonshot.md §2): A, the three JIS English modes and JIS Kana. Ids have gaps:
    // the menu lists them by index; the retired 1, 2, 9, 10 and unknown ids are refused, which the
    // shell turns into Mode A (SetMode).
    bool listed_kana = false;
    for (std::uint32_t i = 0; i < jk_mode_count(); ++i) listed_kana |= jk_mode_id_at(i) == JK_MODE_A_KANA;
    Check(listed_kana && jk_mode_id_at(jk_mode_count()) == UINT32_MAX, "jk_mode_id_at lists JIS Kana");
    {
        JkEngine* e = jk_new();
        Check(jk_set_mode(e, JK_MODE_A_KANA) == 1 && jk_set_mode(e, 4242) == 0, "unknown mode ids are refused");
        for (std::uint32_t retired : {1u, 2u, 9u, 10u}) {
            Check(jk_set_mode(e, retired) == 0, "the retired modes are refused");
        }
        jk_free(e);
    }
    // F6 shows the staged keys as hiragana in Mode A until the commit (docs/moonshot.md §4); the
    // かな key (変換) forces Korean. From JIS Kana, 変換 switches the engine back to Mode A itself:
    // the shell follows jk_mode after each call.
    {
        JkEngine* e = jk_new();
        jk_set_mode(e, JK_MODE_A);
        for (const Press& p : {g, k, s}) jk_key(e, jk::EngineKey(p.vk, p.scan, false), 0);
        jk_key(e, JK_KEY_F6, 0);
        Check(jk_mode(e) == JK_MODE_A && jk_script(e) == JK_SCRIPT_HIRAGANA && Str(jk_preedit_ptr(e), jk_preedit_len(e)) == "きのと",
              "F6: G K S as きのと, still Mode A");
        Check(jk_kana_key(e) == 1 && jk_mode(e) == JK_MODE_A && Str(jk_preedit_ptr(e), jk_preedit_len(e)) == "한",
              "変換 after F6: 한 again");
        jk_flush(e);
        jk_set_mode(e, JK_MODE_A_KANA);
        for (const Press& p : {g, k, s}) jk_key(e, jk::EngineKey(p.vk, p.scan, false), 0);
        Check(Str(jk_preedit_ptr(e), jk_preedit_len(e)) == "きのと", "JIS Kana: G K S as きのと");
        Check(jk_kana_key(e) == 1 && jk_mode(e) == JK_MODE_A && Str(jk_preedit_ptr(e), jk_preedit_len(e)) == "한",
              "変換 from JIS Kana: Mode A, 한 again");
        jk_free(e);
    }

    // The training-file reader (training.cpp): the repo's example, escapes, and bad input.
    {
        std::ifstream file(JK_DOCS_DIR "/training-example.json", std::ios::binary);
        std::stringstream text;
        text << file.rdbuf();
        const auto example = jk::json::Parse(text.str());
        const jk::json::Value* format = example ? example->Get("format") : nullptr;
        Check(format && format->String() &&
                  (*format->String() == "jis-moonshot-training" || *format->String() == "jis-korean-training"),
              "docs/training-example.json parses");
        Check(example && example->Get("version") && example->Get("version")->Number() == 1.0, "version 1");
        const auto escaped = jk::json::Parse(R"({"a": "\uD55C\u0041\n", "b": [1, -2.5e1, true, null]})");
        Check(escaped && *escaped->Get("a")->String() == "\xED\x95\x9C" "A\n", "\\u escapes");
        Check(escaped && escaped->Get("b")->array.size() == 4 && escaped->Get("b")->array[1].number == -25,
              "arrays and numbers");
        Check(!jk::json::Parse("{\"a\": }") && !jk::json::Parse("[1,") && !jk::json::Parse("\"\\uD800\""),
              "bad input is rejected");
        std::string deep(1000, '[');
        Check(!jk::json::Parse(deep), "deep nesting is rejected, not a stack overflow");
    }

    // ABC's English layouts (spec §3.6.2, §3.6.4, §8.4): the shell types jk_legend_in.
    {
        Check(!jk_mode_is_english(JK_MODE_A) && !jk_mode_is_english(JK_MODE_A_KANA) && !jk_mode_is_english(4242),
              "Mode A and JIS Kana aren't English");
        const std::uint32_t digit2 = jk::EngineKey(0x32, 0x03, false), key_s = jk::EngineKey(0x53, 0x1F, false);
        Check(jk_mode_is_english(3) && jk_legend_in(digit2, 1, 0, 3) == U'"', "JIS English: Shift+2 is \"");
        Check(jk_mode_is_english(7) && jk_legend_in(key_s, 0, 0, 7) == U'o', "JIS Dvorak: the S key is o");
        Check(jk_mode_is_english(8) && jk_legend_in(key_s, 0, 0, 8) == U'r', "JIS Colemak: the S key is r");
        Check(!jk_mode_is_english(9) && !jk_mode_is_english(10), "US and UK English are retired");
    }

    // The log truncates an overlong line instead of ending the process (log.cpp).
    {
        wchar_t line[16];
        const std::wstring big(100, L'x');
        size_t n = Format(line, 16, 3, L"%s", big.c_str());
        Check(n == 15 && line[14] == L'\n', "an overlong log line is truncated");
        n = Format(line, 16, 0, L"ab%d", 5);
        Check(n == 4 && std::wstring(line, n) == L"ab5\n", "a log line that fits");
    }

    std::printf(failures ? "%d failed\n" : "all passed\n", failures);
    return failures ? 1 : 0;
}
