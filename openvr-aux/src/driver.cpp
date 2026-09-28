#include "debug_udp.hpp"
#include <openvr_driver.h>
#include <cstdio>
#include <cstring>
#include <chrono>
#include <filesystem>
#include <fstream>
#include <string>
#include <windows.h>

extern "C" __declspec(dllexport) void* __cdecl HmdDriverFactory(const char*, int*);

namespace qptp {
static void Log(const char* message) { vr::VRDriverLog()->Log(message); }
static void LogError(const char* action, const char* path, int code) {
    char line[256];
    std::snprintf(line, sizeof(line), "[qptp] %s %s error=%d", action, path, code);
    Log(line);
}
static bool AutoLaunchEnabled() {
    wchar_t roaming[MAX_PATH]{};
    const DWORD length = GetEnvironmentVariableW(L"APPDATA", roaming, MAX_PATH);
    if (!length || length >= MAX_PATH) return true;
    const auto file = std::filesystem::path(roaming) / L"dev.lateir.qptp" / L"steamvr_lifecycle";
    std::ifstream input(file, std::ios::binary);
    char value = '1';
    if (input.get(value) && value == '0') return false;
    return true;
}
static void LaunchInstalledApp() {
    if (!AutoLaunchEnabled()) {
        Log("[qptp] SteamVR lifecycle disabled; skipping application autostart");
        return;
    }
    HMODULE module = nullptr;
    if (!GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
        reinterpret_cast<LPCWSTR>(&HmdDriverFactory), &module)) {
        LogError("GetModuleHandleExW", "", static_cast<int>(GetLastError()));
        return;
    }
    wchar_t path[MAX_PATH]{};
    const DWORD length = GetModuleFileNameW(module, path, MAX_PATH);
    if (!length || length == MAX_PATH) { LogError("GetModuleFileNameW", "", static_cast<int>(GetLastError())); return; }
    const auto app = std::filesystem::path(path).parent_path().parent_path().parent_path().parent_path() / L"qptp.exe";
    if (!std::filesystem::exists(app)) {
        Log("[qptp] Installed application not found beside driver; skipping autostart");
        return;
    }
    std::wstring command = L"\"" + app.wstring() + L"\" --steamvr-autostart";
    STARTUPINFOW startup{}; startup.cb = sizeof(startup);
    PROCESS_INFORMATION process{};
    const auto directory = app.parent_path().wstring();
    if (!CreateProcessW(app.c_str(), command.data(), nullptr, nullptr, FALSE, 0, nullptr,
        directory.c_str(), &startup, &process)) {
        LogError("CreateProcessW", "qptp.exe", static_cast<int>(GetLastError()));
        return;
    }
    CloseHandle(process.hThread);
    CloseHandle(process.hProcess);
    Log("[qptp] Installed application launched with SteamVR");
}
static bool Property(vr::PropertyContainerHandle_t container, vr::ETrackedDeviceProperty property, const char* value) {
    const auto error = vr::VRProperties()->SetStringProperty(container, property, value);
    if (error != vr::TrackedProp_Success) LogError("property", value, error);
    return error == vr::TrackedProp_Success;
}
static vr::DriverPose_t StationaryPose(bool connected, bool valid) {
    vr::DriverPose_t pose{};
    pose.qWorldFromDriverRotation.w = 1; pose.qDriverFromHeadRotation.w = 1; pose.qRotation.w = 1;
    pose.deviceIsConnected = connected; pose.poseIsValid = connected && valid;
    pose.result = pose.poseIsValid ? vr::TrackingResult_Running_OK : vr::TrackingResult_Uninitialized;
    return pose;
}
struct PadHandles {
    vr::VRInputComponentHandle_t x = vr::k_ulInvalidInputComponentHandle;
    vr::VRInputComponentHandle_t y = vr::k_ulInvalidInputComponentHandle;
    vr::VRInputComponentHandle_t touch = vr::k_ulInvalidInputComponentHandle;
    vr::VRInputComponentHandle_t force = vr::k_ulInvalidInputComponentHandle;
};
struct HandHandles {
    PadHandles pad;
    vr::VRInputComponentHandle_t single_button = vr::k_ulInvalidInputComponentHandle;
    vr::VRInputComponentHandle_t double_button_1 = vr::k_ulInvalidInputComponentHandle;
    vr::VRInputComponentHandle_t double_button_2 = vr::k_ulInvalidInputComponentHandle;
    vr::VRInputComponentHandle_t stylus = vr::k_ulInvalidInputComponentHandle;
    vr::VRInputComponentHandle_t trigger_proximity = vr::k_ulInvalidInputComponentHandle;
    vr::VRInputComponentHandle_t trigger_slide = vr::k_ulInvalidInputComponentHandle;
};
// Single Treadmill-role input device. Only Left/Right/Treadmill roles get a /user path that bindings can use,
// and Treadmill is the one that works alongside the real hand controllers.
class AuxiliaryDevice final : public vr::ITrackedDeviceServerDriver {
    uint32_t id_ = vr::k_unTrackedDeviceIndexInvalid;
    // Keep a stationary valid pose so SteamVR can use this auxiliary input, parked 9001 m above the origin
    // (as OpenVR-SpaceCalibrator hides devices) so applications that enumerate controllers never see it.
    vr::DriverPose_t pose_ = [] { auto pose = StationaryPose(true, true); pose.vecPosition[1] = 9001; return pose; }();
    HandHandles left_, right_;
    bool components_ok_ = true;
    std::chrono::steady_clock::time_point last_log_{};
    void Scalar(vr::PropertyContainerHandle_t container, const char* path, vr::VRInputComponentHandle_t& handle, vr::EVRScalarUnits units) {
        const auto error = vr::VRDriverInput()->CreateScalarComponent(container, path, &handle, vr::VRScalarType_Absolute, units);
        if (error != vr::VRInputError_None) { LogError("CreateScalarComponent", path, error); components_ok_ = false; }
        else { char line[160]; std::snprintf(line, sizeof(line), "[qptp] Input component created: %s", path); Log(line); }
    }
    void Boolean(vr::PropertyContainerHandle_t container, const char* path, vr::VRInputComponentHandle_t& handle) {
        const auto error = vr::VRDriverInput()->CreateBooleanComponent(container, path, &handle);
        if (error != vr::VRInputError_None) { LogError("CreateBooleanComponent", path, error); components_ok_ = false; }
        else { char line[160]; std::snprintf(line, sizeof(line), "[qptp] Input component created: %s", path); Log(line); }
    }
    void CreateHand(vr::PropertyContainerHandle_t container, const char* side, HandHandles& hand) {
        char path[96];
        auto make = [&](const char* suffix) { std::snprintf(path, sizeof(path), "/input/%s_%s", side, suffix); };
        make("pad/x"); Scalar(container, path, hand.pad.x, vr::VRScalarUnits_NormalizedTwoSided);
        make("pad/y"); Scalar(container, path, hand.pad.y, vr::VRScalarUnits_NormalizedTwoSided);
        make("pad/touch"); Boolean(container, path, hand.pad.touch);
        make("pad/force"); Scalar(container, path, hand.pad.force, vr::VRScalarUnits_NormalizedOneSided);
        make("single_button/click"); Boolean(container, path, hand.single_button);
        make("double_button_1/click"); Boolean(container, path, hand.double_button_1);
        make("double_button_2/click"); Boolean(container, path, hand.double_button_2);
        make("stylus/value"); Scalar(container, path, hand.stylus, vr::VRScalarUnits_NormalizedOneSided);
        make("trigger_proximity/value"); Scalar(container, path, hand.trigger_proximity, vr::VRScalarUnits_NormalizedOneSided);
        make("trigger_slide/value"); Scalar(container, path, hand.trigger_slide, vr::VRScalarUnits_NormalizedOneSided);
    }
    void UpdateScalar(vr::VRInputComponentHandle_t handle, float value) {
        const auto error = vr::VRDriverInput()->UpdateScalarComponent(handle, value, 0);
        if (error != vr::VRInputError_None) LogError("UpdateScalarComponent", "", error);
    }
    void UpdateBoolean(vr::VRInputComponentHandle_t handle, bool value) {
        const auto error = vr::VRDriverInput()->UpdateBooleanComponent(handle, value, 0);
        if (error != vr::VRInputError_None) LogError("UpdateBooleanComponent", "", error);
    }
    void UpdateHand(const HandHandles& h, const AddonState& s) {
        UpdateScalar(h.pad.x, s.pad.x); UpdateScalar(h.pad.y, s.pad.y);
        UpdateBoolean(h.pad.touch, s.pad.touch); UpdateScalar(h.pad.force, s.pad.force);
        UpdateBoolean(h.single_button, s.single_button);
        UpdateBoolean(h.double_button_1, s.double_button_1);
        UpdateBoolean(h.double_button_2, s.double_button_2);
        UpdateScalar(h.stylus, s.stylus);
        UpdateScalar(h.trigger_proximity, s.trigger_proximity);
        UpdateScalar(h.trigger_slide, s.trigger_slide);
    }
public:
    vr::EVRInitError Activate(uint32_t id) override {
        id_ = id;
        char line[128]; std::snprintf(line, sizeof(line), "[qptp] Device activated index=%u", id); Log(line);
        const auto container = vr::VRProperties()->TrackedDeviceToPropertyContainer(id);
        components_ok_ = true;
        components_ok_ &= Property(container, vr::Prop_ModelNumber_String, "QPTP Auxiliary Input");
        components_ok_ &= Property(container, vr::Prop_ManufacturerName_String, "QPTP");
        components_ok_ &= Property(container, vr::Prop_TrackingSystemName_String, "qptp");
        components_ok_ &= Property(container, vr::Prop_RegisteredDeviceType_String, "qptp/qptp_input");
        components_ok_ &= Property(container, vr::Prop_ControllerType_String, "qptp");
        components_ok_ &= Property(container, vr::Prop_RenderModelName_String, "qptp");
        components_ok_ &= Property(container, vr::Prop_InputProfilePath_String, "{qptp}/input/qptp_profile.json");
        const auto role_error = vr::VRProperties()->SetInt32Property(container, vr::Prop_ControllerRoleHint_Int32, vr::TrackedControllerRole_Treadmill);
        if (role_error != vr::TrackedProp_Success) { LogError("role property", "Treadmill", role_error); components_ok_ = false; }
        Log("[qptp] Input profile path set: {qptp}/input/qptp_profile.json");
        CreateHand(container, "left", left_);
        CreateHand(container, "right", right_);
        vr::VRServerDriverHost()->TrackedDevicePoseUpdated(id_, pose_, sizeof(pose_));
        return components_ok_ ? vr::VRInitError_None : vr::VRInitError_Driver_Failed;
    }
    void Deactivate() override { Log("[qptp] Device deactivated"); id_ = vr::k_unTrackedDeviceIndexInvalid; }
    void EnterStandby() override {}
    void* GetComponent(const char*) override { return nullptr; }
    void DebugRequest(const char*, char* response, uint32_t size) override { if (size) response[0] = 0; }
    vr::DriverPose_t GetPose() override { return pose_; }
    void RunFrame(const InputState& state, bool changed) {
        if (id_ == vr::k_unTrackedDeviceIndexInvalid || !components_ok_) return;
        UpdateHand(left_, state.left); UpdateHand(right_, state.right);
        if (changed && (last_log_ == std::chrono::steady_clock::time_point{} ||
            std::chrono::steady_clock::now() - last_log_ >= std::chrono::seconds(1))) {
            last_log_ = std::chrono::steady_clock::now();
            const auto& l = state.left; const auto& r = state.right;
            char line[384];
            std::snprintf(line, sizeof(line), "[qptp] Input update L=(%.2f,%.2f,t%d,f%.2f,s%d,d%d,d%d,st%.2f,tp%.2f,ts%.2f) R=(%.2f,%.2f,t%d,f%.2f,s%d,d%d,d%d,st%.2f,tp%.2f,ts%.2f)",
                l.pad.x, l.pad.y, l.pad.touch, l.pad.force, l.single_button, l.double_button_1, l.double_button_2, l.stylus, l.trigger_proximity, l.trigger_slide,
                r.pad.x, r.pad.y, r.pad.touch, r.pad.force, r.single_button, r.double_button_1, r.double_button_2, r.stylus, r.trigger_proximity, r.trigger_slide);
            Log(line);
        }
    }
};
// Input-less device that only shows one Quest Pro controller's battery in SteamVR. OptOut keeps it out of hand selection.
class BatteryDevice final : public vr::ITrackedDeviceServerDriver {
    uint32_t id_ = vr::k_unTrackedDeviceIndexInvalid;
    const char* side_;
    bool connected_ = false;
    float battery_ = -1;
    // Never a valid pose, so games cannot place anything at this device's position.
    static vr::DriverPose_t BatteryPose(bool connected) { return StationaryPose(connected, false); }
public:
    explicit BatteryDevice(const char* side) : side_(side) {}
    vr::EVRInitError Activate(uint32_t id) override {
        id_ = id;
        char line[128]; std::snprintf(line, sizeof(line), "[qptp] %s battery device activated index=%u", side_, id); Log(line);
        const auto container = vr::VRProperties()->TrackedDeviceToPropertyContainer(id);
        const bool left = std::strcmp(side_, "left") == 0;
        auto* props = vr::VRProperties();
        bool ok = Property(container, vr::Prop_ModelNumber_String, left ? "QPTP Left Battery" : "QPTP Right Battery");
        ok &= Property(container, vr::Prop_ManufacturerName_String, "QPTP");
        ok &= Property(container, vr::Prop_TrackingSystemName_String, "qptp");
        ok &= Property(container, vr::Prop_RegisteredDeviceType_String, left ? "qptp/qptp_left_battery" : "qptp/qptp_right_battery");
        ok &= Property(container, vr::Prop_ControllerType_String, "qptp_battery");
        ok &= Property(container, vr::Prop_RenderModelName_String, "qptp");
        ok &= Property(container, vr::Prop_InputProfilePath_String, "{qptp}/input/qptp_battery_profile.json");
        ok &= props->SetInt32Property(container, vr::Prop_ControllerRoleHint_Int32, vr::TrackedControllerRole_OptOut) == vr::TrackedProp_Success;
        ok &= props->SetInt32Property(container, vr::Prop_ControllerHandSelectionPriority_Int32, -1) == vr::TrackedProp_Success;
        ok &= props->SetBoolProperty(container, vr::Prop_NeverTracked_Bool, true) == vr::TrackedProp_Success;
        ok &= props->SetBoolProperty(container, vr::Prop_DeviceIsWireless_Bool, true) == vr::TrackedProp_Success;
        ok &= props->SetBoolProperty(container, vr::Prop_DeviceIsCharging_Bool, false) == vr::TrackedProp_Success;
        ok &= props->SetBoolProperty(container, vr::Prop_DeviceProvidesBatteryStatus_Bool, false) == vr::TrackedProp_Success;
        if (!ok) Log(left ? "[qptp] left battery device properties failed" : "[qptp] right battery device properties failed");
        vr::VRServerDriverHost()->TrackedDevicePoseUpdated(id_, BatteryPose(false), sizeof(vr::DriverPose_t));
        return ok ? vr::VRInitError_None : vr::VRInitError_Driver_Failed;
    }
    void Deactivate() override { id_ = vr::k_unTrackedDeviceIndexInvalid; }
    void EnterStandby() override {}
    void* GetComponent(const char*) override { return nullptr; }
    void DebugRequest(const char*, char* response, uint32_t size) override { if (size) response[0] = 0; }
    vr::DriverPose_t GetPose() override { return BatteryPose(connected_); }
    // Shown as connected only while the app reports a battery level for this controller.
    void RunFrame(const AddonState& state) {
        if (id_ == vr::k_unTrackedDeviceIndexInvalid) return;
        const auto container = vr::VRProperties()->TrackedDeviceToPropertyContainer(id_);
        if (state.battery_valid != connected_) {
            connected_ = state.battery_valid;
            if (!connected_) battery_ = -1;
            vr::VRProperties()->SetBoolProperty(container, vr::Prop_DeviceProvidesBatteryStatus_Bool, connected_);
            vr::VRServerDriverHost()->TrackedDevicePoseUpdated(id_, BatteryPose(connected_), sizeof(vr::DriverPose_t));
            char line[96]; std::snprintf(line, sizeof(line), "[qptp] %s battery device %s", side_, connected_ ? "connected" : "disconnected"); Log(line);
        }
        if (connected_ && state.battery != battery_) {
            battery_ = state.battery;
            vr::VRProperties()->SetFloatProperty(container, vr::Prop_DeviceBatteryPercentage_Float, battery_);
        }
    }
};
class Provider final : public vr::IServerTrackedDeviceProvider {
    AuxiliaryDevice device_;
    BatteryDevice left_battery_{"left"}, right_battery_{"right"};
    DebugUdpInputSource* source_ = nullptr;
    InputState state_{};
public:
    vr::EVRInitError Init(vr::IVRDriverContext* context) override {
        VR_INIT_SERVER_DRIVER_CONTEXT(context);
        Log("[qptp] Driver initialized");
        source_ = new DebugUdpInputSource();
        Log(source_->ready() ? "[qptp] Debug UDP listening on 127.0.0.1:39571" : "[qptp] Debug UDP failed to bind port 39571");
        const bool added = vr::VRServerDriverHost()->TrackedDeviceAdded("qptp_input", vr::TrackedDeviceClass_Controller, &device_);
        Log(added ? "[qptp] Device registered serial=qptp_input" : "[qptp] Device registration failed");
        // Battery devices are optional: failing to add them must not disable the input device.
        const bool left_battery = vr::VRServerDriverHost()->TrackedDeviceAdded("qptp_left_battery", vr::TrackedDeviceClass_Controller, &left_battery_);
        const bool right_battery = vr::VRServerDriverHost()->TrackedDeviceAdded("qptp_right_battery", vr::TrackedDeviceClass_Controller, &right_battery_);
        Log(left_battery && right_battery ? "[qptp] Battery devices registered serial=qptp_left_battery,qptp_right_battery" : "[qptp] Battery device registration failed");
        if (added) LaunchInstalledApp();
        return added ? vr::VRInitError_None : vr::VRInitError_Driver_Failed;
    }
    void Cleanup() override {
        delete source_; source_ = nullptr;
        Log("[qptp] Driver cleanup");
        VR_CLEANUP_SERVER_DRIVER_CONTEXT();
    }
    const char* const* GetInterfaceVersions() override { return vr::k_InterfaceVersions; }
    void RunFrame() override {
        const bool changed = source_ && source_->Poll(state_);
        device_.RunFrame(state_, changed);
        left_battery_.RunFrame(state_.left);
        right_battery_.RunFrame(state_.right);
        vr::VREvent_t event{};
        while (vr::VRServerDriverHost()->PollNextEvent(&event, sizeof(event))) {}
    }
    bool ShouldBlockStandbyMode() override { return false; }
    void EnterStandby() override {}
    void LeaveStandby() override {}
};
}
static qptp::Provider provider;
extern "C" __declspec(dllexport) void* __cdecl HmdDriverFactory(const char* name, int* error) {
    if (name && std::strcmp(name, vr::IServerTrackedDeviceProvider_Version) == 0) {
        if (error) *error = vr::VRInitError_None;
        return &provider;
    }
    if (error) *error = vr::VRInitError_Init_InterfaceNotFound;
    return nullptr;
}
