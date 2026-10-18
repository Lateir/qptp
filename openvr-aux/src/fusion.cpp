// Controller wrapping follows QFTPlus src/steamvr/driver.cpp (MIT, QproFaceTracking contributors).
#include "fusion.hpp"
#include <openvr_driver.h>
#include <windows.h>
#include <cstdio>
#include <memory>
#include <mutex>
#include <vector>

namespace qptp::fusion {
namespace {
using namespace vr;
constexpr VRInputComponentHandle_t kNone = k_ulInvalidInputComponentHandle;
struct Hand {
    PropertyContainerHandle_t container = k_ulInvalidPropertyContainer;
    VRInputComponentHandle_t x = kNone, y = kNone, touch = kNone, force = kNone, click = kNone;
    VRInputComponentHandle_t single = kNone, upper = kNone, lower = kNone, stylus = kNone, proximity = kNone, slide = kNone;
    bool battery_shown = false; float battery = -1;
};
std::mutex g_lock;
Hand g_hands[2];
bool g_conflict = false;
bool g_quest3_is_pro = false;
bool g_quest3[2] = {false, false}; // A hand controller reported a Quest 3 model, adopted or not.

void Log(const char* message) { VRDriverLog()->Log(message); }

void Scalar(PropertyContainerHandle_t c, const char* path, VRInputComponentHandle_t* handle, EVRScalarUnits units) {
    if (VRDriverInput()->CreateScalarComponent(c, path, handle, VRScalarType_Absolute, units) != VRInputError_None) *handle = kNone;
}
void Boolean(PropertyContainerHandle_t c, const char* path, VRInputComponentHandle_t* handle) {
    if (VRDriverInput()->CreateBooleanComponent(c, path, handle) != VRInputError_None) *handle = kNone;
}

// Takes over a Quest Pro controller that the Oculus/Steam Link driver just activated.
void Adopt(uint32_t id) {
    CVRPropertyHelpers* props = VRProperties();
    const PropertyContainerHandle_t c = props->TrackedDeviceToPropertyContainer(id);
    const std::string model = props->GetStringProperty(c, Prop_RenderModelName_String);
    const std::string profile = props->GetStringProperty(c, Prop_InputProfilePath_String);
    const int role = props->GetInt32Property(c, Prop_ControllerRoleHint_Int32);
    char line[384];
    std::snprintf(line, sizeof(line), "[qptp] Fusion sees controller %u: model %s, profile %s, role %d", id, model.c_str(), profile.c_str(), role); Log(line);
    // Quest Pro first; a Quest 3 model ("quest_plus") may be Quest Pro hardware behind Virtual Desktop.
    const bool pro = model.find("quest_pro") != std::string::npos, quest3 = model.find("quest_plus") != std::string::npos;
    if (!pro && !quest3) return;
    const bool hand_role = role == TrackedControllerRole_LeftHand || role == TrackedControllerRole_RightHand;
    if (quest3 && hand_role) {
        { std::lock_guard<std::mutex> hold(g_lock); g_quest3[role == TrackedControllerRole_RightHand] = true; }
        if (!g_quest3_is_pro) { std::snprintf(line, sizeof(line), "[qptp] Fusion: %s looks like a Quest 3 controller; waiting for the app to confirm it is Quest Pro", model.c_str()); Log(line); return; }
    }
    if (profile.rfind("{qftplus}", 0) == 0) {
        { std::lock_guard<std::mutex> hold(g_lock); g_conflict = true; }
        std::snprintf(line, sizeof(line), "[qptp] Fusion left %s alone: QFTPlus already owns it", model.c_str()); Log(line);
        return;
    }
    if (profile != "{oculus}/input/touch_profile.json" || (role != TrackedControllerRole_LeftHand && role != TrackedControllerRole_RightHand)) {
        std::snprintf(line, sizeof(line), "[qptp] Fusion left %s alone (input profile %s, role %d)", model.c_str(), profile.c_str(), role); Log(line);
        return;
    }
    const int side = role == TrackedControllerRole_RightHand;
    props->SetStringProperty(c, Prop_InputProfilePath_String, "{qptp}/input/qptp_quest_pro_profile.json");
    props->SetStringProperty(c, Prop_ControllerType_String, "qptp_quest_pro");
    // Confirmed Quest Pro hardware that reported a Quest 3 model also gets the Quest Pro render model.
    if (quest3) props->SetStringProperty(c, Prop_RenderModelName_String, side ? "oculus_quest_pro_controller_right" : "oculus_quest_pro_controller_left");
    Hand hand; hand.container = c;
    Scalar(c, "/input/trackpad/x", &hand.x, VRScalarUnits_NormalizedTwoSided);
    Scalar(c, "/input/trackpad/y", &hand.y, VRScalarUnits_NormalizedTwoSided);
    Boolean(c, "/input/trackpad/touch", &hand.touch);
    Scalar(c, "/input/trackpad/force", &hand.force, VRScalarUnits_NormalizedOneSided);
    Boolean(c, "/input/trackpad/click", &hand.click);
    Boolean(c, "/input/pad_button/click", &hand.single);
    Boolean(c, "/input/pad_upper/click", &hand.upper);
    Boolean(c, "/input/pad_lower/click", &hand.lower);
    Scalar(c, "/input/stylus/value", &hand.stylus, VRScalarUnits_NormalizedOneSided);
    Scalar(c, "/input/trigger_proximity/value", &hand.proximity, VRScalarUnits_NormalizedOneSided);
    Scalar(c, "/input/trigger_slide/value", &hand.slide, VRScalarUnits_NormalizedOneSided);
    { std::lock_guard<std::mutex> hold(g_lock); g_hands[side] = hand; }
    std::snprintf(line, sizeof(line), "[qptp] Fusion adopted %s %s%s", side ? "right" : "left", model.c_str(), quest3 ? " as Quest Pro" : ""); Log(line);
}

class Shim final : public ITrackedDeviceServerDriver {
    ITrackedDeviceServerDriver* inner_;
    uint32_t id_ = k_unTrackedDeviceIndexInvalid;
public:
    explicit Shim(ITrackedDeviceServerDriver* inner) : inner_(inner) {}
    EVRInitError Activate(uint32_t id) override {
        const EVRInitError result = inner_->Activate(id);
        if (result == VRInitError_None) { id_ = id; Adopt(id); }
        return result;
    }
    void Deactivate() override {
        const auto container = VRProperties()->TrackedDeviceToPropertyContainer(id_);
        { std::lock_guard<std::mutex> hold(g_lock); for (auto& hand : g_hands) if (hand.container == container) hand = Hand{}; }
        id_ = k_unTrackedDeviceIndexInvalid;
        inner_->Deactivate();
    }
    void EnterStandby() override { inner_->EnterStandby(); }
    void* GetComponent(const char* name) override { return inner_->GetComponent(name); }
    void DebugRequest(const char* request, char* response, uint32_t size) override { inner_->DebugRequest(request, response, size); }
    DriverPose_t GetPose() override { return inner_->GetPose(); }
};

// IVRServerDriverHost::TrackedDeviceAdded is the first virtual method. On x64 a member call passes `this`
// as the first argument, so a free function with the host pointer first can stand in for it.
using AddedFn = bool (*)(IVRServerDriverHost*, const char*, ETrackedDeviceClass, ITrackedDeviceServerDriver*);
AddedFn g_added = nullptr;
void** g_slot = nullptr;
std::vector<std::unique_ptr<Shim>> g_shims;

bool Added(IVRServerDriverHost* host, const char* serial, ETrackedDeviceClass kind, ITrackedDeviceServerDriver* driver) {
    if (kind == TrackedDeviceClass_Controller && driver != nullptr) {
        std::lock_guard<std::mutex> hold(g_lock);
        driver = g_shims.emplace_back(std::make_unique<Shim>(driver)).get();
    }
    return g_added(host, serial, kind, driver);
}
bool Patch(void* target) {
    DWORD old = 0;
    if (!VirtualProtect(g_slot, sizeof(void*), PAGE_READWRITE, &old)) return false;
    g_slot[0] = target;
    VirtualProtect(g_slot, sizeof(void*), old, &old);
    return true;
}
}

bool Start(bool quest3_is_pro) {
    g_quest3_is_pro = quest3_is_pro;
    IVRServerDriverHost* host = VRServerDriverHost();
    if (host == nullptr) return false;
    g_slot = *reinterpret_cast<void***>(host);
    g_added = reinterpret_cast<AddedFn>(g_slot[0]);
    if (!Patch(reinterpret_cast<void*>(&Added))) { g_slot = nullptr; g_added = nullptr; return false; }
    Log("[qptp] Fusion (beta) waiting for Quest Pro controllers");
    return true;
}

void Stop() {
    if (g_slot != nullptr && g_slot[0] == reinterpret_cast<void*>(&Added)) Patch(reinterpret_cast<void*>(g_added));
    g_slot = nullptr;
    std::lock_guard<std::mutex> hold(g_lock);
    for (auto& hand : g_hands) hand = Hand{};
    g_shims.clear();
}

void RunFrame(const InputState& state) {
    Hand hands[2];
    { std::lock_guard<std::mutex> hold(g_lock); hands[0] = g_hands[0]; hands[1] = g_hands[1]; }
    IVRDriverInput* input = VRDriverInput();
    auto scalar = [&](VRInputComponentHandle_t handle, float value) { if (handle != kNone) input->UpdateScalarComponent(handle, value, 0); };
    auto boolean = [&](VRInputComponentHandle_t handle, bool value) { if (handle != kNone) input->UpdateBooleanComponent(handle, value, 0); };
    for (int side = 0; side < 2; side++) {
        const Hand& h = hands[side];
        if (h.container == k_ulInvalidPropertyContainer) continue;
        const AddonState& s = side ? state.right : state.left;
        scalar(h.x, s.pad.x); scalar(h.y, s.pad.y); boolean(h.touch, s.pad.touch); scalar(h.force, s.pad.force); boolean(h.click, s.pad.click);
        boolean(h.single, s.single_button); boolean(h.upper, s.double_button_1); boolean(h.lower, s.double_button_2);
        scalar(h.stylus, s.stylus); scalar(h.proximity, s.trigger_proximity); scalar(h.slide, s.trigger_slide);
        // Battery goes on the real controller; erase it only if QPTP was the one that set it.
        auto* props = VRProperties();
        if (s.battery_valid && (!h.battery_shown || s.battery != h.battery)) {
            props->SetFloatProperty(h.container, Prop_DeviceBatteryPercentage_Float, s.battery);
            props->SetBoolProperty(h.container, Prop_DeviceProvidesBatteryStatus_Bool, true);
            std::lock_guard<std::mutex> hold(g_lock);
            if (g_hands[side].container == h.container) { g_hands[side].battery_shown = true; g_hands[side].battery = s.battery; }
        } else if (!s.battery_valid && h.battery_shown) {
            props->EraseProperty(h.container, Prop_DeviceBatteryPercentage_Float);
            props->SetBoolProperty(h.container, Prop_DeviceProvidesBatteryStatus_Bool, false);
            std::lock_guard<std::mutex> hold(g_lock);
            if (g_hands[side].container == h.container) g_hands[side].battery_shown = false;
        }
    }
}

std::string Report() {
    std::lock_guard<std::mutex> hold(g_lock);
    const bool left = g_hands[0].container != k_ulInvalidPropertyContainer, right = g_hands[1].container != k_ulInvalidPropertyContainer;
    return std::string("left=") + (left ? "1" : "0") + " right=" + (right ? "1" : "0") + " q3left=" + (g_quest3[0] ? "1" : "0") + " q3right=" + (g_quest3[1] ? "1" : "0") + " conflict=" + (g_conflict ? "qftplus" : "none");
}
}
