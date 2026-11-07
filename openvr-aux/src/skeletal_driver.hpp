#pragma once
#include "input_state.hpp"
#include <openvr_driver.h>
#include <string>
namespace qptp::skeleton {
bool Start(vr::IVRDriverInput* input, void (*log)(const char*) = nullptr, vr::IVRDriverInput* legacy = nullptr, vr::IVRDriverInput* latest = nullptr);
void Stop();
void Adopt(vr::PropertyContainerHandle_t container,int side);
void Remove(vr::PropertyContainerHandle_t container);
void Frame(const InputState& state);
std::string Report();
bool Debug(vr::PropertyContainerHandle_t container, char* response, uint32_t size);
}
