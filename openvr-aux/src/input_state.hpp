#pragma once
namespace qptp {
struct TouchpadState { float x = 0, y = 0, force = 0; bool touch = false; };
struct AddonState { TouchpadState pad; bool extra1 = false, extra2 = false; };
struct InputState { AddonState left, right; };
class IInputSource {
public:
    virtual ~IInputSource() = default;
    virtual bool Poll(InputState& state) = 0;
};
}
