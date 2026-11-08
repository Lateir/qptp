#ifdef NDEBUG
#undef NDEBUG
#endif
#include "skeletal_driver.hpp"
#include "skeletal_mapping.hpp"
#include "hand_simulation.h"
#include <MinHook.h>
#include <cassert>
#include <windows.h>
#include <algorithm>
#include <cmath>
#include <cstring>
#include <iostream>
#include <map>
#include <functional>
#include <future>
#include <atomic>
#include <thread>
#include <mutex>
using namespace vr;
using namespace qptp;
using namespace qptp::skeleton;
struct Mock:IVRDriverInput {
    uint64_t next=1;int writes=0;bool fail=false;EVRSkeletalTrackingLevel last_level=VRSkeletalTracking_Estimated;
    std::map<uint64_t,Pose> poses;
    std::function<void()> on_write;
    EVRInputError CreateBooleanComponent(PropertyContainerHandle_t,const char*,VRInputComponentHandle_t* h)override{*h=next++;return VRInputError_None;}
    EVRInputError UpdateBooleanComponent(VRInputComponentHandle_t,bool,double)override{return VRInputError_None;}
    EVRInputError CreateScalarComponent(PropertyContainerHandle_t,const char*,VRInputComponentHandle_t* h,EVRScalarType,EVRScalarUnits)override{*h=next++;return VRInputError_None;}
    EVRInputError UpdateScalarComponent(VRInputComponentHandle_t,float,double)override{return VRInputError_None;}
    EVRInputError CreateHapticComponent(PropertyContainerHandle_t,const char*,VRInputComponentHandle_t*)override{return VRInputError_None;}
    EVRInputError CreateSkeletonComponent(PropertyContainerHandle_t,const char*,const char*,const char*,EVRSkeletalTrackingLevel level,const VRBoneTransform_t*,uint32_t,VRInputComponentHandle_t* h)override{last_level=level;*h=next++;return VRInputError_None;}
    EVRInputError UpdateSkeletonComponent(VRInputComponentHandle_t h,EVRSkeletalMotionRange,const VRBoneTransform_t* p,uint32_t n)override{
        if(on_write)on_write();
        ++writes;if(fail)return VRInputError_InvalidHandle;assert(n==31);std::copy_n(p,n,poses[h].begin());return VRInputError_None;
    }
    EVRInputError CreatePoseComponent(PropertyContainerHandle_t,const char*,VRInputComponentHandle_t*)override{return VRInputError_None;}
    EVRInputError UpdatePoseComponent(VRInputComponentHandle_t,const HmdMatrix34_t*,double)override{return VRInputError_None;}
    EVRInputError CreateEyeTrackingComponent(PropertyContainerHandle_t,const char*,VRInputComponentHandle_t*)override{return VRInputError_None;}
    EVRInputError UpdateEyeTrackingComponent(VRInputComponentHandle_t,const VREyeTrackingData_t*,double)override{return VRInputError_None;}
};
struct Legacy final:Mock {
    EVRInputError CreateBooleanComponent(PropertyContainerHandle_t c,const char* p,VRInputComponentHandle_t* h)override{return Mock::CreateBooleanComponent(c,p,h);}
};
// Keep virtual dispatch: testing the same vtable boundary used by another driver.
__declspec(noinline) void native_pose(IVRDriverInput* api,uint64_t h,const Pose& p,int range=0){api->UpdateSkeletonComponent(h,static_cast<EVRSkeletalMotionRange>(range),p.data(),31);}
__declspec(noinline) uint64_t native_skeleton(IVRDriverInput* api,uint64_t c){uint64_t h;assert(api->CreateSkeletonComponent(c,"/input/skeleton/left","/skeleton/hand/left","/pose/raw",VRSkeletalTracking_Estimated,nullptr,0,&h)==VRInputError_None);return h;}
__declspec(noinline) uint64_t scalar(IVRDriverInput* api,uint64_t c,const char* path){uint64_t h;api->CreateScalarComponent(c,path,&h,VRScalarType_Absolute,VRScalarUnits_NormalizedOneSided);return h;}
__declspec(noinline) void update(IVRDriverInput* api,uint64_t h,float v){api->UpdateScalarComponent(h,v,0);}
bool equal(const Pose& a,const Pose& b){return std::memcmp(a.data(),b.data(),sizeof(Pose))==0;}
using ScalarUpdate=EVRInputError(*)(IVRDriverInput*,VRInputComponentHandle_t,float,double);
ScalarUpdate late_scalar_original=nullptr;
IVRDriverInput* late_wrapper=nullptr;
std::mutex late_transport_mutex;
bool late_nested=false;
EVRInputError late_scalar_hook(IVRDriverInput* self,VRInputComponentHandle_t h,float v,double t){
    auto result=late_scalar_original(self,h,v,t);
    if(result!=VRInputError_None)return result;
    // Virtual Desktop forwards to its saved original, locks its own state, then sends
    // another scalar via the cached _004 original. Re-entering its hook here is fatal.
    if(!late_transport_mutex.try_lock())throw std::runtime_error("transport mutex re-entry");
    std::lock_guard<std::mutex> lock(late_transport_mutex,std::adopt_lock);
    if(!late_nested){late_nested=true;late_scalar_original(late_wrapper,h,v,t);late_nested=false;}
    return result;
}
struct V {float x,y,z;};
V operator+(V a,V b){return {a.x+b.x,a.y+b.y,a.z+b.z};}
V operator-(V a,V b){return {a.x-b.x,a.y-b.y,a.z-b.z};}
V operator*(V a,float s){return {a.x*s,a.y*s,a.z*s};}
float dot(V a,V b){return a.x*b.x+a.y*b.y+a.z*b.z;}
V cross(V a,V b){return {a.y*b.z-a.z*b.y,a.z*b.x-a.x*b.z,a.x*b.y-a.y*b.x};}
V norm(V a){return a*(1/std::sqrt(dot(a,a)));}
V pos(const VRBoneTransform_t& b){return {b.position.v[0],b.position.v[1],b.position.v[2]};}
HmdQuaternionf_t product(HmdQuaternionf_t a,HmdQuaternionf_t b){return {a.w*b.w-a.x*b.x-a.y*b.y-a.z*b.z,a.w*b.x+a.x*b.w+a.y*b.z-a.z*b.y,a.w*b.y-a.x*b.z+a.y*b.w+a.z*b.x,a.w*b.z+a.x*b.y-a.y*b.x+a.z*b.w};}
V tip(const Pose& p,int first,int last){V v{};HmdQuaternionf_t q{1,0,0,0};for(int i=first;i<=last;++i){auto b=pos(p[i]);auto r=product(product(q,{0,b.x,b.y,b.z}),{q.w,-q.x,-q.y,-q.z});v=v+V{r.x,r.y,r.z};q=product(q,p[i].orientation);}return v;}
int main(){
    for(bool right:{false,true})for(bool with:{false,true}){
        auto base=Base(right,with,{});
        FingerState f;f.valid=true;f.age_ms=0;f.contact=true;f.proximity=0.8f;f.slide=0.7f;
        for(float x:{-1.f,0.f,1.f})for(float y:{-1.f,0.f,1.f}){
            f.x=x;f.y=y;auto p=Map(base,right,with,{},f);
            for(size_t i=0;i<31;++i){auto q=p[i].orientation;
                assert(std::abs(q.w*q.w+q.x*q.x+q.y*q.y+q.z*q.z-1)<0.0001f);
                for(float v:p[i].position.v)assert(std::isfinite(v));
                if(i<2 || (i>=11 && i<=25) || i>=28)assert(std::memcmp(&p[i],&base[i],sizeof(p[i]))==0);
            }
        }
        auto translated=base;translated[1].position.v[0]+=1.f;Auxiliaries(translated);
        for(int i=26;i<31;++i)assert(std::abs(translated[i].position.v[0]-base[i].position.v[0]-1.f)<0.0001f);
        auto moved=Map(base,right,with,{},f);assert(!equal(base,moved));
        // Hardware fixture: both resting and pressed proximity saturate at 1.
        f.proximity=1;
        auto resting=Map(base,right,with,{},f);
        auto pressing=Map(base,right,with,{1,0,true,false},f);
        assert(!equal(resting,pressing));
        f.contact=false;auto no_thumb=Map(base,right,with,{},f);for(int i:{2,3,4,5,26})assert(std::memcmp(&no_thumb[i],&base[i],sizeof(base[i]))==0);
        f.age_ms=500;assert(equal(base,Map(base,right,with,{},f)));
        f.age_ms=0;f.proximity=NAN;assert(equal(base,Map(base,right,with,{},f)));
        // Slide uses the trigger flexion axis, with no abduction or axial twist.
        auto contact=Base(right,with,{0,0,false,true});
        f={0,0,.8f,0,0,true,true};auto low=Map(contact,right,with,{0,0,true,false},f);f.slide=1;auto high=Map(contact,right,with,{0,0,true,false},f);
        float curl=.8f*.35f*(with?.65f:1.f);
        f.slide=.5f;auto neutral=Map(contact,right,with,{0,0,true,false},f);
        Pose expected_low{};MyHandSimulation simulation;
        simulation.ComputeSkeletonTransforms(right?TrackedControllerRole_RightHand:TrackedControllerRole_LeftHand,{0,curl+1.f/6.f,0,0,0},{},expected_low.data());
        Pose expected_high{};simulation.ComputeSkeletonTransforms(right?TrackedControllerRole_RightHand:TrackedControllerRole_LeftHand,{0,curl-1.f/3.f,0,0,0},{},expected_high.data());
        for(auto pair:{std::make_pair(low,expected_low),std::make_pair(high,expected_high)}){
            auto a=pair.first[7].orientation,b=pair.second[7].orientation;
            assert(std::abs(a.w*b.w+a.x*b.x+a.y*b.y+a.z*b.z)> .99999f);
        }
        auto q=neutral[7].orientation;HmdQuaternionf_t inv{q.w,-q.x,-q.y,-q.z};
        auto outward=product(high[7].orientation,inv),inward=product(low[7].orientation,inv);
        assert(std::abs(std::abs(outward.w)-std::cos(3.14159265f/12))<1e-5f);
        assert(std::abs(std::abs(inward.w)-std::cos(3.14159265f/24))<1e-5f);
        for(int i:{6,8,9,10})assert(std::memcmp(&low[i],&high[i],sizeof(low[i]))==0);
        // No-contact slide endpoints must be identical to centered slide.
        f.slide=.5f;auto lifted=Map(contact,right,with,{},f);
        for(float slide:{0.f,1.f}){f.slide=slide;auto endpoint=Map(contact,right,with,{},f);
            for(int i=6;i<=10;++i)assert(std::memcmp(&lifted[i],&endpoint[i],sizeof(lifted[i]))==0);}
        // X rotates only the thumb base: all relative descendant rotations stay fixed,
        // so the whole Y-curl plane follows X instead of IK changing the finger shape.
        f.slide=.5f;
        V across=norm(pos(contact[6])-pos(contact[21]));if(right)across=across*-1.f;
        V forward=pos(contact[11]);forward=norm(forward-across*dot(forward,across));
        for(float y:{-1.f,0.f,1.f}){
            f.x=-1;f.y=y;auto left=Map(contact,right,with,{},f);
            f.x=1;auto rightward=Map(contact,right,with,{},f);
            assert(std::memcmp(&left[2].orientation,&rightward[2].orientation,sizeof(left[2].orientation))!=0);
            for(int i=3;i<=5;++i)assert(std::memcmp(&left[i],&rightward[i],sizeof(left[i]))==0);
            V displacement=tip(rightward,2,5)-tip(left,2,5);
            assert(std::abs(dot(displacement,across))<1e-5f);
            V normal=norm(cross(forward,across));
            assert(std::abs(dot(displacement,normal))>.005f);
            assert(std::sqrt(dot(displacement,displacement))>.01f);
            for(int i=2;i<=5;++i)for(int j=0;j<4;++j)assert(left[i].position.v[j]==contact[i].position.v[j]);
        }
        f.x=0;f.y=-1;auto curled=Map(contact,right,with,{},f);
        f.y=1;auto extended=Map(contact,right,with,{},f);
        V vertical=tip(extended,2,5)-tip(curled,2,5);assert(std::sqrt(dot(vertical,vertical))>.01f);
    }
    Mock mock;IVRDriverInput* api=&mock;
    using SavedUpdate=EVRInputError(*)(IVRDriverInput*,VRInputComponentHandle_t,EVRSkeletalMotionRange,const VRBoneTransform_t*,uint32_t);
    auto saved_update=reinterpret_cast<SavedUpdate>((*reinterpret_cast<void***>(api))[6]);
    auto base=Base(false,true,{});
    Mock cloned;cloned.next=4000;
    void* clone_table[11]{};std::copy_n(*reinterpret_cast<void***>(&cloned),11,clone_table);
    *reinterpret_cast<void***>(&cloned)=clone_table;
    assert(Start(api));
    auto foreign=native_skeleton(&cloned,2000);native_pose(&cloned,foreign,base);
    assert(cloned.writes==1 && equal(cloned.poses[foreign],base));
    auto h=native_skeleton(api,42),other=native_skeleton(api,99);
    native_pose(api,h,base);native_pose(api,h,base,1);
    Adopt(42,0);InputState s;s.skeleton_enabled=true;s.left.finger={0.8f,-0.8f,0.9f,0.8f,0,true,true};
    const int before=mock.writes;native_pose(api,h,base);assert(mock.writes==before);
    native_pose(api,other,base);assert(mock.writes==before+1);
    Frame(s);assert(!equal(mock.poses[h],base));assert(Report().find("skelleft=active")!=std::string::npos);
    s.skeleton_enabled=false;Frame(s);assert(equal(mock.poses[h],base));
    s.skeleton_enabled=true;s.left.finger.valid=false;Frame(s);assert(equal(mock.poses[h],base));
    mock.fail=true;Frame(s);assert(Report().find("skelleft=error")!=std::string::npos);mock.fail=false;
    auto changed=Base(false,true,{0.7f,0.8f,true,true});native_pose(api,h,changed);assert(equal(mock.poses[h],changed));
    Remove(42);native_pose(api,h,base);assert(equal(mock.poses[h],base));
    auto trigger=scalar(api,43,"/input/trigger/value"),grip=scalar(api,43,"/input/grip/value");
    update(api,trigger,0.8f);update(api,grip,0.5f);Adopt(43,1);assert(mock.last_level==VRSkeletalTracking_Partial);s.skeleton_enabled=false;Frame(s);
    assert(Report().find("skelright=disabled")!=std::string::npos);
    auto own=mock.next-1;assert(equal(mock.poses[own],Base(true,false,{0.8f,0.5f,false,false})));
    // Virtual Desktop's shim may begin writing AFTER qptp created the component.
    const int own_before=mock.writes;
    saved_update(api,own,VRSkeletalMotionRange_WithController,base.data(),31);
    saved_update(api,own,VRSkeletalMotionRange_WithoutController,base.data(),31);
    assert(mock.writes==own_before); // No foreign pose may race our frame writer.
    Frame(s);assert(equal(mock.poses[own],base));
    Remove(43);Adopt(43,1);Frame(s); // Reconnect clears old controls and handles.
    Stop();const int stopped=mock.writes;native_pose(api,h,base);assert(mock.writes==stopped+1);
    Legacy legacy;legacy.next=1000;
    assert(Start(api,nullptr,&legacy));
    auto legacy_handle=native_skeleton(&legacy,77);native_pose(&legacy,legacy_handle,base);native_pose(&legacy,legacy_handle,base,1);
    Adopt(77,0);s.skeleton_enabled=false;Frame(s); // Both interface versions share SteamVR's global handle space.
    Stop();
    // Reproduce SteamVR's _004 wrapper forwarding to a distinct _005 object.
    // This catches missed native writers AND recursive publication through the wrapper.
    Mock latest;latest.next=2000;
    assert(!Start(api,nullptr,nullptr,&latest)); // Unknown ABI must be rejected.
    auto* stubs=static_cast<unsigned char*>(VirtualAlloc(nullptr,7*16,MEM_COMMIT|MEM_RESERVE,PAGE_READWRITE));assert(stubs);
    void* table[7]{};
    for(int slot=0;slot<7;++slot){
        const unsigned char prefix[]={0x48,0x8b,0x49,0x08};
        auto code=stubs+slot*16;std::memset(code,0xcc,16);std::memcpy(code,prefix,4);
        if(slot==0){const unsigned char tail[]={0x48,0x8b,0x01,0x48,0xff,0x20};std::memcpy(code+4,tail,sizeof(tail));}
        else if(slot==1 || slot==3 || slot==4){const unsigned char tail[]={0x48,0x8b,0x01,0x48,0xff,0x60,static_cast<unsigned char>(slot*8)};std::memcpy(code+4,tail,sizeof(tail));}
        else {const unsigned char tail[]={0x4c,0x8b,0x11,0x49,0xff,0x62,static_cast<unsigned char>(slot*8)};std::memcpy(code+4,tail,sizeof(tail));}
        table[slot]=code;
    }
    DWORD old;assert(VirtualProtect(stubs,7*16,PAGE_EXECUTE_READ,&old));FlushInstructionCache(GetCurrentProcess(),stubs,7*16);
    struct Wrapper {void** table;IVRDriverInput* inner;} wrapper{table,&latest};
    auto* wrapper_api=reinterpret_cast<IVRDriverInput*>(&wrapper);
    Mock queried_latest;
    assert(Start(wrapper_api,nullptr,nullptr,&queried_latest)); // Same ABI/table, separate factory instance.
    auto scalar_handle=scalar(wrapper_api,900,"/input/trigger/value");
    assert(scalar_handle==2000 && latest.next==2001 && queried_latest.next==1);
    void* scalar_target=(*reinterpret_cast<void***>(wrapper_api))[3];
    assert(MH_CreateHook(scalar_target,reinterpret_cast<void*>(&late_scalar_hook),reinterpret_cast<void**>(&late_scalar_original))==MH_OK);
    assert(MH_EnableHook(scalar_target)==MH_OK);late_wrapper=wrapper_api;
    bool transport_failed=false;try{update(wrapper_api,scalar_handle,.4f);}catch(const std::exception&){transport_failed=true;}
    assert(MH_DisableHook(scalar_target)==MH_OK);assert(MH_RemoveHook(scalar_target)==MH_OK);
    late_scalar_original=nullptr;late_wrapper=nullptr;late_nested=false;
    assert(!transport_failed);
    auto latest_handle=native_skeleton(&latest,900);
    native_pose(&latest,latest_handle,base);native_pose(&latest,latest_handle,base,1);
    Adopt(900,0);s.skeleton_enabled=true;s.left.finger.valid=true;Frame(s);
    assert(!equal(latest.poses[latest_handle],base));
    Adopt(901,1);Frame(s); // Own creation must not be misclassified as a native writer.
    s.skeleton_enabled=false;Frame(s);assert(equal(latest.poses[latest_handle],base));
    Stop();VirtualFree(stubs,0,MEM_RELEASE);
    // Regression for the real crash: native input owns the runtime lock and calls
    // our hook while frame publication needs that runtime lock. No API call may
    // retain our state mutex, or these threads deadlock and SteamVR's watchdog aborts.
    assert(Start(api));auto concurrent=native_skeleton(api,1001);
    native_pose(api,concurrent,base);native_pose(api,concurrent,base,1);Adopt(1001,0);
    std::mutex runtime_lock;std::promise<void> ready,publishing;
    auto ready_future=ready.get_future(),publishing_future=publishing.get_future();std::atomic_bool signaled=false;
    mock.on_write=[&]{if(!signaled.exchange(true))publishing.set_value();std::lock_guard<std::mutex> lock(runtime_lock);};
    std::thread native([&]{std::lock_guard<std::mutex> lock(runtime_lock);ready.set_value();publishing_future.wait();native_pose(api,concurrent,base);});
    ready_future.wait();Frame(s);native.join();mock.on_write={};Stop();
    std::cout<<"Skeleton mapping and driver lifecycle checks passed\n";
}
