#pragma once
#include "input_state.hpp"
#include <chrono>
#include <string>
#include <utility>
#include <winsock2.h>
namespace qptp {
class DebugUdpInputSource final : public IInputSource {
    SOCKET socket_ = INVALID_SOCKET;
    bool winsock_ = false;
    std::chrono::steady_clock::time_point last_packet_{};
    std::string acknowledgement_ = "QPTP1";
public:
    DebugUdpInputSource();
    ~DebugUdpInputSource() override;
    bool Poll(InputState& state) override;
    bool ready() const { return socket_ != INVALID_SOCKET; }
    // Sent back for every packet, so the app can show which driver mode is running.
    void set_acknowledgement(std::string text) { acknowledgement_ = std::move(text); }
};
}
