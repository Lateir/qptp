#pragma once
#include <string>
namespace qptp {
struct TouchpadState { float x = 0, y = 0, force = 0; bool touch = false, click = false; };
struct FingerState { float x = 0, y = 0, proximity = 0, slide = 0, age_ms = 1000000; bool valid = false, contact = false; };
struct AddonState { FingerState finger; TouchpadState pad; bool single_button = false, double_button_1 = false, double_button_2 = false; float stylus = 0, trigger_proximity = 0, trigger_slide = 0, battery = 0; bool battery_valid = false; };
struct InputState { bool skeleton_enabled = false; AddonState left, right; };
class IInputSource {
public:
    virtual ~IInputSource() = default;
    virtual bool Poll(InputState& state) = 0;
};
}
