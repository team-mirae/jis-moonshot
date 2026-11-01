#include "shared_data.h"

#include <windows.h>

#include <mutex>
#include <new>

#include "dll.h"
#include "settings.h"
#include "log.h"
#include "resource.h"

namespace jk {
namespace {

#ifdef JK_PREDICT
struct Bytes {
    const std::uint8_t* data = nullptr;
    size_t size = 0;
};

Bytes Resource(int id) {
    HRSRC info = FindResourceW(DllInstance(), MAKEINTRESOURCEW(id), RT_RCDATA);
    if (!info) return {};
    HGLOBAL handle = LoadResource(DllInstance(), info);
    if (!handle) return {};
    return {static_cast<const std::uint8_t*>(LockResource(handle)), SizeofResource(DllInstance(), info)};
}

std::once_flag g_predictor_once;
JkPredictor* g_predictor = nullptr;
#endif

}  // namespace

bool RunInBackground(void (*job)()) {
    // A reference on the DLL that the thread pool drops only after the callback has returned
    // (FreeLibraryWhenCallbackReturns): DllRelease alone, before returning through DLL code,
    // let the DLL unload under it.
    HMODULE module = nullptr;
    if (!GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
                            reinterpret_cast<LPCWSTR>(&RunInBackground), &module)) {
        return false;
    }
    struct Job {
        void (*run)();
        HMODULE module;
    };
    auto* context = new (std::nothrow) Job{job, module};
    DllAddRef();  // DllCanUnloadNow says no meanwhile
    const BOOL queued = context && TrySubmitThreadpoolCallback(
        [](PTP_CALLBACK_INSTANCE instance, void* context) {
            auto* job = static_cast<Job*>(context);
            const HMODULE module = job->module;
            job->run();
            delete job;
            DllRelease();
            FreeLibraryWhenCallbackReturns(instance, module);
        },
        context, nullptr);
    if (!queued) {
        delete context;
        DllRelease();
        FreeLibrary(module);
    }
    return queued != FALSE;
}

void Preload() {
#ifdef JK_PREDICT
    static std::once_flag once;
    std::call_once(once, [] {
        // Predictions are always on (docs/moonshot.md §6). A program excluded from predictions
        // never parses the model.
        RunInBackground([] {
            if (settings::AppPredicting(settings::ExeName())) SharedPredictor();
        });
    });
#endif
}

#ifdef JK_PREDICT
const JkPredictor* SharedPredictor() {
    std::call_once(g_predictor_once, [] {
        const ULONGLONG start = GetTickCount64();
        const Bytes model = Resource(IDR_RANKER);
        const Bytes en = Resource(IDR_COMMON_EN);
        const Bytes ko = Resource(IDR_COMMON_KO);
        const Bytes nouns = Resource(IDR_NOUNS);
        if (model.data && en.data && ko.data) {
            g_predictor = jk_predictor_new(model.data, model.size, en.data, en.size, ko.data,
                                           ko.size, nouns.data, nouns.size);
        }
#ifdef JK_LEARNED
        // The learned ranker (spec §10.13); without its model the rules rank.
        const Bytes learned = Resource(IDR_LEARNED);
        if (g_predictor && learned.data) {
            if (JkPredictor* ranked = jk_predictor_with_learned(g_predictor, learned.data, learned.size)) {
                jk_predictor_free(g_predictor);
                g_predictor = ranked;
                log::Write(L"learned ranker loaded (%zu bytes)", learned.size);
            } else {
                log::Write(L"learned ranker unavailable: the rules rank");
            }
        }
#endif
        // The kana reading's lexicon (docs/moonshot.md §5): without it, no kana reading.
        const Bytes japanese = Resource(IDR_COMMON_JA);
        if (g_predictor && japanese.data) {
            if (JkPredictor* kana = jk_predictor_with_japanese(g_predictor, japanese.data, japanese.size)) {
                jk_predictor_free(g_predictor);
                g_predictor = kana;
            } else {
                log::Write(L"Japanese lexicon unavailable: no kana reading");
            }
        }
        // Kana-kanji conversion (docs/moonshot.md §5.1): without it, no kanji.
        const Bytes kanji = Resource(IDR_KANJI);
        if (g_predictor && kanji.data) {
            if (JkPredictor* converts = jk_predictor_with_kanji(g_predictor, kanji.data, kanji.size)) {
                jk_predictor_free(g_predictor);
                g_predictor = converts;
            } else {
                log::Write(L"kanji data unavailable: no kanji");
            }
        }
        if (!g_predictor) log::Write(L"predictor unavailable");
        log::Write(L"predictor loaded in %llu ms", GetTickCount64() - start);
    });
    return g_predictor;
}
#endif

void ReleaseSharedData() {
    // DLL unload: no engine is left (DllCanUnloadNow) and no background job runs (it holds a
    // module reference until it has returned).
#ifdef JK_PREDICT
    if (g_predictor) jk_predictor_free(g_predictor);
    g_predictor = nullptr;
#endif
}

}  // namespace jk
