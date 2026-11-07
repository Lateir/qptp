#pragma once
#include "input_state.hpp"
#include <openvr_driver.h>
#include <array>
namespace qptp::skeleton {
constexpr size_t BoneCount = 31;
inline constexpr char MappingRevision[] = "index-contact-thumb-transverse-inverted-v8";
using Pose = std::array<vr::VRBoneTransform_t, BoneCount>;
struct Controls { float trigger = 0, grip = 0; bool trigger_touch = false, thumb_touch = false; };
Pose Base(bool right, bool with_controller, const Controls& controls);
// Returns a new pose; never accumulates offsets into the source animation.
Pose Map(const Pose& base, bool right, bool with_controller, const Controls& controls, const FingerState& finger);
Pose Blend(const Pose& a, const Pose& b, float weight);
void Auxiliaries(Pose& pose, bool thumb_only = false);
}
