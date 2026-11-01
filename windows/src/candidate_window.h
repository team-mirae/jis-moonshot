// The list under the text: the predictions while typing (spec §10). A small topmost popup that never takes focus; a click picks a row. Excluded from
// screen capture.
#pragma once

#include <windows.h>

#include <functional>
#include <string>
#include <vector>

namespace jk {

class CandidateWindow {
public:
    struct Row {
        std::wstring label;  // a row number ("1"…"9"); empty for predictions
        std::wstring text;
        std::wstring comment;
        // A Japanese reading (kanji, kana): drawn in a Japanese font, so kanji take their Japanese
        // (shinjitai) forms whatever the system language (docs/moonshot.md §5.1).
        bool japanese = false;
    };

    CandidateWindow() = default;
    ~CandidateWindow();
    CandidateWindow(const CandidateWindow&) = delete;
    CandidateWindow& operator=(const CandidateWindow&) = delete;

    // Shows `rows` below `anchor` (screen coordinates of the text), or above it when there's no
    // room below. `selected` is highlighted (-1: none); `footer` goes under the rows. A click on
    // row i calls `on_pick(i)`.
    void Show(std::vector<Row> rows, int selected, std::wstring footer, const RECT& anchor,
              std::function<void(size_t)> on_pick);
    void Hide();
    // Unregisters the window class (DLL unload; fails harmlessly while a list window exists).
    static void Unregister();

private:
    static LRESULT CALLBACK Proc(HWND hwnd, UINT message, WPARAM wparam, LPARAM lparam);
    void Paint(HDC dc);
    void UpdateFont();

    HWND hwnd_ = nullptr;
    HFONT font_ = nullptr;
    HFONT japanese_font_ = nullptr;
    UINT font_dpi_ = 0;
    HFONT FontFor(const Row& row) const { return row.japanese && japanese_font_ ? japanese_font_ : font_; }
    std::vector<Row> rows_;
    int selected_ = -1;
    std::wstring footer_;
    std::function<void(size_t)> on_pick_;
    int row_height_ = 0;
    int label_width_ = 0;
    int text_width_ = 0;
    int padding_ = 0;
};

}  // namespace jk
