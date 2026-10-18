#pragma once
#include "input_state.hpp"
#include <string>
// Fusion (beta): adds the QPTP pad and buttons to the Quest Pro controllers themselves instead of a Treadmill device.
namespace qptp::fusion {
// Quest Pro controllers are adopted automatically. Controllers that report a Quest 3 model (Virtual Desktop can do this
// for Quest Pro hardware) are only adopted once the user confirmed in the app that they are Quest Pro controllers.
bool Start(bool quest3_is_pro);     // Hooks controller creation; false means Classic must be used instead.
void Stop();                        // Restores the hook.
void RunFrame(const InputState& state);
std::string Report();               // "left=1 right=0 q3left=0 q3right=0 conflict=none" for the app's status line.
}
