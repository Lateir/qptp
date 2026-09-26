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
struct PadHandles {
    vr::VRInputComponentHandle_t x = vr::k_ulInvalidInputComponentHandle;
    vr::VRInputComponentHandle_t y = vr::k_ulInvalidInputComponentHandle;
    vr::VRInputComponentHandle_t touch = vr::k_ulInvalidInputComponentHandle;
    vr::VRInputComponentHandle_t force = vr::k_ulInvalidInputComponentHandle;
};
struct HandHandles {
    PadHandles pad;
    vr::VRInputComponentHandle_t extra1 = vr::k_ulInvalidInputComponentHandle;
    vr::VRInputComponentHandle_t extra2 = vr::k_ulInvalidInputComponentHandle;
};
class AuxiliaryDevice final : public vr::ITrackedDeviceServerDriver {
    uint32_t id_ = vr::k_unTrackedDeviceIndexInvalid;
    vr::DriverPose_t pose_{};
    HandHandles left_, right_;
    bool components_ok_ = true;
    std::chrono::steady_clock::time_point last_log_{};
    bool Property(vr::PropertyContainerHandle_t container, vr::ETrackedDeviceProperty property, const char* value) {
        const auto error = vr::VRProperties()->SetStringProperty(container, property, value);
        if (error != vr::TrackedProp_Success) LogError("property", value, error);
        return error == vr::TrackedProp_Success;
    }
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
        make("extra_1/click"); Boolean(container, path, hand.extra1);
        make("extra_2/click"); Boolean(container, path, hand.extra2);
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
        UpdateBoolean(h.extra1, s.extra1); UpdateBoolean(h.extra2, s.extra2);
    }
public:
    AuxiliaryDevice() {
        pose_.qWorldFromDriverRotation.w = 1;
        pose_.qDriverFromHeadRotation.w = 1;
        pose_.qRotation.w = 1;
        pose_.deviceIsConnected = true;
        pose_.poseIsValid = false;
        pose_.result = vr::TrackingResult_Uninitialized;
    }
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
            char line[256];
            std::snprintf(line, sizeof(line), "[qptp] Input update L=(%.2f,%.2f,t%d,f%.2f,b%d,b%d) R=(%.2f,%.2f,t%d,f%.2f,b%d,b%d)",
                state.left.pad.x, state.left.pad.y, state.left.pad.touch, state.left.pad.force, state.left.extra1, state.left.extra2,
                state.right.pad.x, state.right.pad.y, state.right.pad.touch, state.right.pad.force, state.right.extra1, state.right.extra2);
            Log(line);
        }
    }
};
class Provider final : public vr::IServerTrackedDeviceProvider {
    AuxiliaryDevice device_;
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
