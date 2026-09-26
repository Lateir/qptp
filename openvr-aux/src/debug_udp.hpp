#pragma once
#include "input_state.hpp"
#include <chrono>
#include <winsock2.h>
namespace qptp {
class DebugUdpInputSource final : public IInputSource {
    SOCKET socket_ = INVALID_SOCKET;
    bool winsock_ = false;
    std::chrono::steady_clock::time_point last_packet_{};
public:
    DebugUdpInputSource();
    ~DebugUdpInputSource() override;
    bool Poll(InputState& state) override;
    bool ready() const { return socket_ != INVALID_SOCKET; }
};
}
