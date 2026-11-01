import Cocoa
import InputMethodKit

// `JISMoonshot mode [A|A_KANA|A_QWERTY…]`: the public API from the command line (spec §8.2), not the input method.
if CommandLine.arguments.count > 1 {
    exit(PublicAPI.runCommand(Array(CommandLine.arguments.dropFirst())))
}

// Input methods are background apps: IMKServer accepts client connections, and the system creates
// one JKInputController per text input client.
let connectionName = Bundle.main.infoDictionary?["InputMethodConnectionName"] as? String
    ?? "me.sjun.inputmethod.JISMoonshot_Connection"
#if PREDICT
/// The predictive layer (spec §10), parsed once and shared. Nil if its files are missing.
let sharedPredictor: OpaquePointer? = {
    func data(_ name: String, _ ext: String) -> Data? {
        Bundle.main.url(forResource: name, withExtension: ext, subdirectory: "predict").flatMap { try? Data(contentsOf: $0) }
    }
    guard let model = data("ranker", "bin"), let english = data("common-en", "txt"), let korean = data("common-ko", "txt")
    else { return nil }
    let nouns = data("english-nouns", "txt") ?? Data()
    let base: OpaquePointer? = model.withUnsafeBytes { m in
        english.withUnsafeBytes { e in
            korean.withUnsafeBytes { k in
                nouns.withUnsafeBytes { n in
                    jk_predictor_new(
                        m.bindMemory(to: UInt8.self).baseAddress, m.count,
                        e.bindMemory(to: UInt8.self).baseAddress, e.count,
                        k.bindMemory(to: UInt8.self).baseAddress, k.count,
                        n.bindMemory(to: UInt8.self).baseAddress, n.count)
                }
            }
        }
    }
    // The learned ranker (spec §10.13). Without its model (or in the legacy build, which ignores
    // it), the rules rank.
    var predictor = base
    if let current = predictor, let learned = data("learned", "bin"),
       let ranked = learned.withUnsafeBytes({ l in jk_predictor_with_learned(current, l.bindMemory(to: UInt8.self).baseAddress, l.count) }) {
        jk_predictor_free(current)
        predictor = ranked
    }
    // The kana reading's lexicon (docs/moonshot.md §5): without it, no kana reading.
    if let current = predictor, let japanese = data("common-ja", "txt"),
       let withKana = japanese.withUnsafeBytes({ j in jk_predictor_with_japanese(current, j.bindMemory(to: UInt8.self).baseAddress, j.count) }) {
        jk_predictor_free(current)
        predictor = withKana
    }
    // Kana-kanji conversion (docs/moonshot.md §5.1): without it, no kanji.
    if let current = predictor, let kanji = data("kanji", "bin"),
       let withKanji = kanji.withUnsafeBytes({ k in jk_predictor_with_kanji(current, k.bindMemory(to: UInt8.self).baseAddress, k.count) }) {
        jk_predictor_free(current)
        predictor = withKanji
    }
    return predictor
}()
#endif

DiagnosticLog.note("launched from \(Bundle.main.bundlePath)")
PublicAPI.start()
#if ENGLISH
// Only if it's of use and already allowed: it never asks for permissions at launch.
KoreanKeyTap.check()
#endif
let server = IMKServer(name: connectionName, bundleIdentifier: Bundle.main.bundleIdentifier)
_ = server
NSApplication.shared.run()
