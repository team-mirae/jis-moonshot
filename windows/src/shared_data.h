// The predictor, parsed once per process from the DLL's resources and shared by every engine
// (it's immutable). Preload starts parsing it on a thread-pool thread when JIS Moonshot activates,
// unless predictions are off in this program (a per-app exclusion); the getter waits for it if
// needed.
#pragma once

#include "jis_korean.h"

namespace jk {

void Preload();
#ifdef JK_PREDICT
const JkPredictor* SharedPredictor();  // nullptr if the model is missing
#endif
// Runs `job` on a thread-pool thread; the DLL stays loaded until it has returned. False: not queued.
bool RunInBackground(void (*job)());
// Frees the shared data: DLL unload only, when no engine is left.
void ReleaseSharedData();

}  // namespace jk
