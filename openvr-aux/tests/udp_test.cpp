#ifdef NDEBUG
#undef NDEBUG
#endif
#include "debug_udp.hpp"
#include <cassert>
#include <chrono>
#include <cstring>
#include <iostream>
#include <thread>
int main(){
    qptp::DebugUdpInputSource source(0);assert(source.ready());
    SOCKET socket=::socket(AF_INET,SOCK_DGRAM,IPPROTO_UDP);assert(socket!=INVALID_SOCKET);
    sockaddr_in addr{};addr.sin_family=AF_INET;addr.sin_addr.s_addr=htonl(INADDR_LOOPBACK);addr.sin_port=htons(source.local_port());
    qptp::InputState state;
    auto send=[&](const char* text){assert(sendto(socket,text,int(std::strlen(text)),0,reinterpret_cast<sockaddr*>(&addr),sizeof(addr))>0);
        std::this_thread::sleep_for(std::chrono::milliseconds(5));assert(source.Poll(state));};
    send("skeleton_enabled=1,left_finger_valid=1,left_finger_age_ms=490,left_finger_contact=1,left_finger_x=2,left_finger_y=-2,left_finger_proximity=1.25,left_finger_slide=-0.25,left_pad_x=0.5");
    assert(state.skeleton_enabled && state.left.finger.valid && state.left.finger.contact);
    assert(state.left.finger.x==1 && state.left.finger.y==-1);
    assert(state.left.finger.proximity==1.25f && state.left.finger.slide==-0.25f);
    std::this_thread::sleep_for(std::chrono::milliseconds(20));source.Poll(state);assert(state.left.finger.age_ms>=500);
    // Legacy datagrams cannot retain a previous app's valid skeleton channel.
    send("left_pad_x=0.2");assert(!state.skeleton_enabled && !state.left.finger.valid);
    send("left_pad_x=nan,left_pad_force=inf");assert(state.left.pad.x==0.2f && state.left.pad.force==0);
    std::this_thread::sleep_for(std::chrono::milliseconds(510));assert(source.Poll(state));assert(state.left.pad.x==0 && !state.left.finger.valid);
    closesocket(socket);std::cout<<"UDP extension compatibility, finite values and freshness checks passed\n";
}
