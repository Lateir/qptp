#include "debug_udp.hpp"
#include <algorithm>
#include <cerrno>
#include <cstdlib>
#include <cstring>
#include <string>
namespace qptp {
DebugUdpInputSource::DebugUdpInputSource() {
    WSADATA data{};
    if (WSAStartup(MAKEWORD(2, 2), &data) != 0) return;
    winsock_ = true;
    socket_ = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP);
    if (socket_ == INVALID_SOCKET) return;
    u_long nonblocking = 1;
    if (ioctlsocket(socket_, FIONBIO, &nonblocking) != 0) { closesocket(socket_); socket_ = INVALID_SOCKET; return; }
    sockaddr_in address{};
    address.sin_family = AF_INET;
    address.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    address.sin_port = htons(39571);
    if (bind(socket_, reinterpret_cast<sockaddr*>(&address), sizeof(address)) != 0) {
        closesocket(socket_); socket_ = INVALID_SOCKET;
    }
}
DebugUdpInputSource::~DebugUdpInputSource() {
    if (socket_ != INVALID_SOCKET) closesocket(socket_);
    if (winsock_) WSACleanup();
}
bool DebugUdpInputSource::Poll(InputState& state) {
    if (!ready()) return false;
    bool received = false;
    char buffer[2049];
    for (;;) {
        sockaddr_in sender{};
        int sender_length = sizeof(sender);
        const int length = recvfrom(socket_, buffer, 2048, 0, reinterpret_cast<sockaddr*>(&sender), &sender_length);
        if (length == SOCKET_ERROR) break;
        if (sender.sin_addr.s_addr != htonl(INADDR_LOOPBACK)) continue;
        static constexpr char acknowledgement[] = "QPTP1";
        sendto(socket_, acknowledgement, sizeof(acknowledgement) - 1, 0,
            reinterpret_cast<sockaddr*>(&sender), sender_length);
        buffer[length] = 0;
        received = true;
        std::string packet(buffer, length);
        size_t start = 0;
        while (start < packet.size()) {
            const auto end = packet.find_first_of("\r\n,", start);
            const auto line = packet.substr(start, end == std::string::npos ? end : end - start);
            const auto equal = line.find('=');
            if (equal != std::string::npos) {
                const auto key = line.substr(0, equal);
                const auto value = line.substr(equal + 1);
                char* tail = nullptr;
                const float number = std::strtof(value.c_str(), &tail);
                if (tail != value.c_str() && *tail == 0) {
                    const auto two = std::clamp(number, -1.f, 1.f);
                    const auto one = std::clamp(number, 0.f, 1.f);
                    const bool on = number != 0;
                    if (key == "left_pad_x") state.left.pad.x = two;
                    else if (key == "left_pad_y") state.left.pad.y = two;
                    else if (key == "left_pad_touch") state.left.pad.touch = on;
                    else if (key == "left_pad_force") state.left.pad.force = one;
                    else if (key == "right_pad_x") state.right.pad.x = two;
                    else if (key == "right_pad_y") state.right.pad.y = two;
                    else if (key == "right_pad_touch") state.right.pad.touch = on;
                    else if (key == "right_pad_force") state.right.pad.force = one;
                    else if (key == "left_single_button") state.left.single_button = on;
                    else if (key == "left_double_button_1") state.left.double_button_1 = on;
                    else if (key == "left_double_button_2") state.left.double_button_2 = on;
                    else if (key == "right_single_button") state.right.single_button = on;
                    else if (key == "right_double_button_1") state.right.double_button_1 = on;
                    else if (key == "right_double_button_2") state.right.double_button_2 = on;
                }
            }
            if (end == std::string::npos) break;
            start = end + 1;
        }
    }
    if (received) last_packet_ = std::chrono::steady_clock::now();
    if (last_packet_ != std::chrono::steady_clock::time_point{} &&
        std::chrono::steady_clock::now() - last_packet_ > std::chrono::seconds(2)) {
        state = {};
        last_packet_ = {};
        return true;
    }
    return received;
}
}
