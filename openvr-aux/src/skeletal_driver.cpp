#include "skeletal_driver.hpp"
#include "skeletal_mapping.hpp"
#include <windows.h>
#include <MinHook.h>
#include <algorithm>
#include <chrono>
#include <cstring>
#include <cstdio>
#include <cmath>
#include <map>
#include <mutex>
#include <vector>

namespace qptp::skeleton {
namespace {
using namespace vr;
using CreateBool=EVRInputError(*)(IVRDriverInput*,PropertyContainerHandle_t,const char*,VRInputComponentHandle_t*);
using UpdateBool=EVRInputError(*)(IVRDriverInput*,VRInputComponentHandle_t,bool,double);
using CreateScalar=EVRInputError(*)(IVRDriverInput*,PropertyContainerHandle_t,const char*,VRInputComponentHandle_t*,EVRScalarType,EVRScalarUnits);
using UpdateScalar=EVRInputError(*)(IVRDriverInput*,VRInputComponentHandle_t,float,double);
using CreateSkeleton=EVRInputError(*)(IVRDriverInput*,PropertyContainerHandle_t,const char*,const char*,const char*,EVRSkeletalTrackingLevel,const VRBoneTransform_t*,uint32_t,VRInputComponentHandle_t*);
using UpdateSkeleton=EVRInputError(*)(IVRDriverInput*,VRInputComponentHandle_t,EVRSkeletalMotionRange,const VRBoneTransform_t*,uint32_t);
IVRDriverInput* g_input=nullptr;
void (*g_log)(const char*)=nullptr;
void** g_table=nullptr;
struct Hook {void** table;int slot;void* original;void* replacement;};
std::vector<Hook> g_hooks;
std::vector<void*> g_detours;
bool g_mh_owned=false;
thread_local unsigned g_passthrough=0;
struct Direct {Direct(){++g_passthrough;}~Direct(){--g_passthrough;}};
std::map<void**,std::array<void*,7>> g_originals;
std::map<void**,void**> g_forward_tables;
std::mutex g_original_mutex;
template<class T>T original(IVRDriverInput*& self,int slot){
    std::lock_guard<std::mutex> lock(g_original_mutex);
    auto** table=*reinterpret_cast<void***>(self);
    auto forward=g_forward_tables.find(table);
    if(forward!=g_forward_tables.end()){
        // A saved _004 thunk dynamically dispatches through the _005 vtable. A later
        // transport hook on our shared callback would then run twice (including while
        // its own mutex is held). Call the validated, snapshotted inner method directly.
        self=*reinterpret_cast<IVRDriverInput**>(reinterpret_cast<unsigned char*>(self)+8);
        table=forward->second;
    }
    auto found=g_originals.find(table);
    if(found!=g_originals.end())return reinterpret_cast<T>(found->second[slot]);
    // Other drivers may clone a dispatch table. Slot 4 is never patched here;
    // use it to identify an unambiguous original instead of throwing across the ABI.
    void* candidate=nullptr;
    for(const auto& entry:g_originals){
        if(entry.second[4]!=table[4])continue;
        if(candidate && candidate!=entry.second[slot])return nullptr;
        candidate=entry.second[slot];
    }
    return reinterpret_cast<T>(candidate);
}
std::mutex g_mutex;
struct Device {
    VRInputComponentHandle_t handle=k_ulInvalidInputComponentHandle;
    int side=-1;
    bool own=false,error=false;
    uint64_t frames=0,native_updates=0;
    FingerState finger;
    Pose base[2]{}, output[2]{};
    bool have[2]{}, blended[2]{};
    std::map<std::string,float> values;
    std::chrono::steady_clock::time_point frame{};
};
struct Component {PropertyContainerHandle_t container;std::string path;};
std::map<PropertyContainerHandle_t,Device> g_devices;
std::map<VRInputComponentHandle_t,Component> g_components;
bool g_enabled=false,g_fresh[2]{};
CreateBool g_create_bool=nullptr;UpdateBool g_update_bool=nullptr;
CreateScalar g_create_scalar=nullptr;UpdateScalar g_update_scalar=nullptr;
CreateSkeleton g_create_skeleton=nullptr;UpdateSkeleton g_update_skeleton=nullptr;

bool exchange(void** table,int slot,void* expected,void* target) {
    DWORD old=0;if(!VirtualProtect(table+slot,sizeof(void*),PAGE_READWRITE,&old))return false;
    auto previous=InterlockedCompareExchangePointer(reinterpret_cast<void* volatile*>(table+slot),target,expected);
    DWORD ignored;VirtualProtect(table+slot,sizeof(void*),old,&ignored);
    return previous==expected;
}
void record(PropertyContainerHandle_t c,const char* path,VRInputComponentHandle_t h) {
    if(g_passthrough || !path)return;
    const std::string p(path);
    if(p=="/input/trigger/value" || p=="/input/grip/value" || p=="/input/trigger/touch" ||
       p=="/input/thumbrest/touch" || p=="/input/joystick/touch" || p=="/input/a/touch" ||
       p=="/input/b/touch" || p=="/input/x/touch" || p=="/input/y/touch") {
        std::lock_guard<std::mutex> lock(g_mutex);g_components[h]={c,p};g_devices[c].values.try_emplace(p,0.f);
    }
}
EVRInputError create_bool(IVRDriverInput* self,PropertyContainerHandle_t c,const char* path,VRInputComponentHandle_t* h) {
    auto fn=original<CreateBool>(self,0);if(!fn)return VRInputError_InvalidHandle;
    auto e=fn(self,c,path,h);if(e==VRInputError_None && h)record(c,path,*h);return e;
}
EVRInputError create_scalar(IVRDriverInput* self,PropertyContainerHandle_t c,const char* path,VRInputComponentHandle_t* h,EVRScalarType t,EVRScalarUnits u) {
    auto fn=original<CreateScalar>(self,2);if(!fn)return VRInputError_InvalidHandle;
    auto e=fn(self,c,path,h,t,u);if(e==VRInputError_None && h)record(c,path,*h);return e;
}
void value(VRInputComponentHandle_t h,float v) {
    if(g_passthrough)return;
    std::lock_guard<std::mutex> lock(g_mutex);auto it=g_components.find(h);
    if(it!=g_components.end())g_devices[it->second.container].values[it->second.path]=std::isfinite(v)?std::clamp(v,0.f,1.f):0.f;
}
EVRInputError update_bool(IVRDriverInput* self,VRInputComponentHandle_t h,bool v,double t) {value(h,v?1.f:0.f);auto fn=original<UpdateBool>(self,1);return fn?fn(self,h,v,t):VRInputError_InvalidHandle;}
EVRInputError update_scalar(IVRDriverInput* self,VRInputComponentHandle_t h,float v,double t) {value(h,v);auto fn=original<UpdateScalar>(self,3);return fn?fn(self,h,v,t):VRInputError_InvalidHandle;}
EVRInputError create_skeleton(IVRDriverInput* self,PropertyContainerHandle_t c,const char* path,const char* skel,const char* pose,EVRSkeletalTrackingLevel level,const VRBoneTransform_t* grip,uint32_t n,VRInputComponentHandle_t* h) {
    auto fn=original<CreateSkeleton>(self,5);if(!fn)return VRInputError_InvalidHandle;
    auto e=fn(self,c,path,skel,pose,level,grip,n,h);
    if(!g_passthrough && e==VRInputError_None && h && skel && (std::strcmp(skel,"/skeleton/hand/left")==0 || std::strcmp(skel,"/skeleton/hand/right")==0)) {
        std::lock_guard<std::mutex> lock(g_mutex);auto& d=g_devices[c];
        d.handle=*h;d.own=false;d.error=false;
    }return e;
}
EVRInputError update_skeleton(IVRDriverInput* self,VRInputComponentHandle_t h,EVRSkeletalMotionRange range,const VRBoneTransform_t* bones,uint32_t n) {
    auto fn=original<UpdateSkeleton>(self,6);if(!fn)return VRInputError_InvalidHandle;
    if(g_passthrough)return fn(self,h,range,bones,n);
    bool suppress=false;int captured_side=-1;
    if(bones && n==BoneCount && (range==VRSkeletalMotionRange_WithController || range==VRSkeletalMotionRange_WithoutController)) {
        std::lock_guard<std::mutex> lock(g_mutex);
        for(auto& entry:g_devices) {auto& d=entry.second;if(d.handle!=h)continue;
            // A transport shim may start producing animation after activation, using the
            // component we just created. It is now a native baseline, not a second writer.
            if(d.own){d.own=false;
                captured_side=d.side;
            }
            ++d.native_updates;
            std::copy_n(bones,BoneCount,d.base[range].begin());d.have[range]=true;
            suppress=d.side>=0 && !d.error;break;
        }
    }
    if(captured_side>=0 && g_log)g_log(captured_side?"[qptp] Skeleton right: late native writer captured":"[qptp] Skeleton left: late native writer captured");
    return suppress?VRInputError_None:fn(self,h,range,bones,n);
}
Controls controls(const Device& d) {
    Controls c;for(auto& v:d.values) {
        if(v.first=="/input/trigger/value")c.trigger=v.second;
        else if(v.first=="/input/grip/value")c.grip=v.second;
        else if(v.first=="/input/trigger/touch")c.trigger_touch=v.second!=0;
        else c.thumb_touch|=v.second!=0;
    }return c;
}
}
// The public SDK still names _004. SteamVR 2.18 forwards its first seven methods
// unchanged to _005. Validate those forwarding instructions and the inner object
// before using that prefix; never assume an unpublished interface has the same ABI.
bool compatible_latest(IVRDriverInput* wrapper,IVRDriverInput* latest){
    auto table=*reinterpret_cast<unsigned char***>(wrapper);
    for(int slot:{0,1,2,3,4,5,6}){
        const unsigned char* p=table[slot];
        const unsigned char prefix[]={0x48,0x8b,0x49,0x08};
        if(std::memcmp(p,prefix,4)!=0){if(g_log){char line[192];std::snprintf(line,sizeof(line),"[qptp] ABI guard slot=%d prefix=%02x %02x %02x %02x",slot,p[0],p[1],p[2],p[3]);g_log(line);}return false;}
        const unsigned char a[]={0x48,0x8b,0x01};
        const unsigned char b[]={0x4c,0x8b,0x11};
        bool valid=false;
        if(std::memcmp(p+4,a,3)==0){
            const int j=p[7]==0x48?8:7; // Optional REX.W on the indirect jump.
            valid=slot==0?(p[j]==0xff && p[j+1]==0x20):(p[j]==0xff && p[j+1]==0x60 && p[j+2]==slot*8);
        }
        if(std::memcmp(p+4,b,3)==0 && (p[7]==0x41 || p[7]==0x49) && p[8]==0xff && p[9]==0x62 && p[10]==slot*8)valid=true;
        if(!valid){if(g_log){char line[192];std::snprintf(line,sizeof(line),"[qptp] ABI guard slot=%d tail=%02x %02x %02x %02x %02x %02x %02x",slot,p[4],p[5],p[6],p[7],p[8],p[9],p[10]);g_log(line);}return false;}
    }
    // Interface factories may return separate instances of the same implementation.
    // Validate the dispatch table used by the verified wrapper, not object identity.
    auto inner=*reinterpret_cast<void***>(reinterpret_cast<char*>(wrapper)+8);
    const bool same=inner && *inner==*reinterpret_cast<void***>(latest);
    if(!same && g_log)g_log("[qptp] ABI guard: forwarded _005 dispatch table differs from queried _005");
    return same;
}
bool Start(IVRDriverInput* input, void (*log)(const char*), IVRDriverInput* legacy,IVRDriverInput* latest) {
    if(g_input || !input)return false;
    g_log=log;
    if(latest && !compatible_latest(input,latest)){
        if(g_log)g_log("[qptp] Unrecognized IVRDriverInput_005 forwarding ABI; skeleton disabled");return false;
    }
    // These indices are specific to IVRDriverInput_004 in our vendored SDK.
    static_assert(sizeof(void*)==8,"Fusion hooks require Windows x64");
    if(std::strcmp(IVRDriverInput_Version,"IVRDriverInput_004")!=0)return false;
    g_table=*reinterpret_cast<void***>(input);
    g_create_bool=reinterpret_cast<CreateBool>(g_table[0]);g_update_bool=reinterpret_cast<UpdateBool>(g_table[1]);
    g_create_scalar=reinterpret_cast<CreateScalar>(g_table[2]);g_update_scalar=reinterpret_cast<UpdateScalar>(g_table[3]);
    g_create_skeleton=reinterpret_cast<CreateSkeleton>(g_table[5]);g_update_skeleton=reinterpret_cast<UpdateSkeleton>(g_table[6]);
    const std::pair<int,void*> targets[]={{0,reinterpret_cast<void*>(&create_bool)},{1,reinterpret_cast<void*>(&update_bool)},
        {2,reinterpret_cast<void*>(&create_scalar)},{3,reinterpret_cast<void*>(&update_scalar)},
        {5,reinterpret_cast<void*>(&create_skeleton)},{6,reinterpret_cast<void*>(&update_skeleton)}};
    std::vector<void**> tables{g_table};
    if(legacy){auto table=*reinterpret_cast<void***>(legacy);if(table!=g_table)tables.push_back(table);}
    if(latest){auto table=*reinterpret_cast<void***>(latest);if(std::find(tables.begin(),tables.end(),table)==tables.end())tables.push_back(table);}
    // v003 and v004 share the first seven methods (verified against the v2.5.1 SDK).
    for(auto table:tables){std::array<void*,7> saved{};std::copy_n(table,7,saved.begin());std::lock_guard<std::mutex> lock(g_original_mutex);g_originals[table]=saved;g_forward_tables.erase(table);}
    if(latest){
        std::lock_guard<std::mutex> lock(g_original_mutex);
        g_forward_tables[g_table]=*reinterpret_cast<void***>(latest);
        if(legacy && compatible_latest(legacy,latest))g_forward_tables[*reinterpret_cast<void***>(legacy)]=*reinterpret_cast<void***>(latest);
    }
    const auto init=MH_Initialize();
    if(init!=MH_OK && init!=MH_ERROR_ALREADY_INITIALIZED){Stop();return false;}
    g_mh_owned=init==MH_OK;
    std::map<void*,void*> trampolines;
    for(auto table:tables){
        void* target=table[6];
        if(!trampolines.count(target)){
            void* trampoline=nullptr;
            if(MH_CreateHook(target,reinterpret_cast<void*>(&update_skeleton),&trampoline)!=MH_OK){Stop();return false;}
            trampolines[target]=trampoline;g_detours.push_back(target);
        }
        g_originals[table][6]=trampolines[target];
    }
    g_update_skeleton=reinterpret_cast<UpdateSkeleton>(g_originals[g_table][6]);
    for(auto table:tables)for(auto& t:targets) {
        void* saved=table[t.first];
        if(!exchange(table,t.first,saved,t.second)){Stop();return false;}
        g_hooks.push_back({table,t.first,saved,t.second});
    }
    for(auto target:g_detours)if(MH_EnableHook(target)!=MH_OK){Stop();return false;}
    g_input=input;if(g_log){g_log("[qptp] Skeleton hooks installed for IVRDriverInput_003/_004 and validated _005, including cached update pointers");g_log(MappingRevision);if(latest)g_log("[qptp] Native input forwarding bypass active (Virtual Desktop re-entry fix)");}return true;
}
void Stop() {
    for(auto target:g_detours)MH_DisableHook(target);
    if(g_table)for(auto it=g_hooks.rbegin();it!=g_hooks.rend();++it)exchange(it->table,it->slot,it->replacement,it->original);
    // No callback should dispatch into a released trampoline after restoration.
    {std::lock_guard<std::mutex> dispatch(g_original_mutex);
        for(auto& hook:g_hooks)if(hook.slot==6)g_originals[hook.table][6]=hook.original;
    }
    for(auto target:g_detours)MH_RemoveHook(target);
    g_detours.clear();if(g_mh_owned){MH_Uninitialize();g_mh_owned=false;}
    std::lock_guard<std::mutex> lock(g_mutex);
    // Retain original dispatch records for calls that loaded a hook pointer just before restoration.
    g_hooks.clear();g_devices.clear();g_components.clear();g_input=nullptr;g_table=nullptr;g_enabled=false;
}
namespace {
struct PendingWrite {PropertyContainerHandle_t container;VRInputComponentHandle_t handle;int range;Pose pose;};
void publish(const std::vector<PendingWrite>& writes,IVRDriverInput* input){
    Direct direct;
    for(const auto& write:writes){
        const auto error=g_update_skeleton(input,write.handle,static_cast<EVRSkeletalMotionRange>(write.range),write.pose.data(),BoneCount);
        if(error!=VRInputError_None){
            {std::lock_guard<std::mutex> lock(g_mutex);auto it=g_devices.find(write.container);
                if(it!=g_devices.end() && it->second.handle==write.handle)it->second.error=true;}
            if(g_log){char line[160];std::snprintf(line,sizeof(line),"[qptp] Skeleton update failed range=%d error=%d",write.range,int(error));g_log(line);}
        }
    }
}
}
void Adopt(PropertyContainerHandle_t c,int side) {
    if(!g_input)return;
    VRInputComponentHandle_t handle;
    {std::lock_guard<std::mutex> lock(g_mutex);auto& d=g_devices[c];d.side=side;handle=d.handle;}
    if(handle!=k_ulInvalidInputComponentHandle){if(g_log)g_log(side?"[qptp] Skeleton right: using native component":"[qptp] Skeleton left: using native component");return;}
    auto grip=Base(side==1,true,{1,1,true,true});
    Direct direct;
    auto e=g_create_skeleton(g_input,c,side?"/input/skeleton/right":"/input/skeleton/left",side?"/skeleton/hand/right":"/skeleton/hand/left",
        "/pose/raw",VRSkeletalTracking_Partial,grip.data(),BoneCount,&handle);
    std::vector<PendingWrite> writes;
    {
        std::lock_guard<std::mutex> lock(g_mutex);auto& d=g_devices[c];d.error=e!=VRInputError_None;
        if(!d.error){d.handle=handle;d.own=true;}
        // Initial data immediately follows creation, including both motion ranges.
        if(d.own)for(int r=0;r<2;++r){d.base[r]=Base(side==1,r==0,controls(d));d.have[r]=true;writes.push_back({c,d.handle,r,d.base[r]});}
    }
    if(g_log){char line[160];std::snprintf(line,sizeof(line),"[qptp] Skeleton %s: create partial component result=%d handle=%llu",side?"right":"left",int(e),static_cast<unsigned long long>(handle));g_log(line);}
    publish(writes,g_input);
}
void Remove(PropertyContainerHandle_t c) {
    std::lock_guard<std::mutex> lock(g_mutex);g_devices.erase(c);
    for(auto it=g_components.begin();it!=g_components.end();)if(it->second.container==c)it=g_components.erase(it);else ++it;
}
void Frame(const InputState& state) {
    if(!g_input)return;
    auto* input=g_input;
    std::vector<PendingWrite> writes;
    {
    std::lock_guard<std::mutex> lock(g_mutex);g_enabled=state.skeleton_enabled;
    g_fresh[0]=state.left.finger.valid && state.left.finger.age_ms<500;
    g_fresh[1]=state.right.finger.valid && state.right.finger.age_ms<500;
    const auto now=std::chrono::steady_clock::now();
    for(auto& entry:g_devices){auto& d=entry.second;if(d.side<0 || d.error || d.handle==k_ulInvalidInputComponentHandle)continue;
        const auto& f=d.side?state.right.finger:state.left.finger;const auto c=controls(d);d.finger=f;++d.frames;
        const float dt=d.frame==std::chrono::steady_clock::time_point{}?1.f:std::chrono::duration<float>(now-d.frame).count();d.frame=now;
        for(int r=0;r<2;++r){
            if(d.own){d.base[r]=Base(d.side==1,r==0,c);d.have[r]=true;}
            if(!d.have[r])continue;
            const bool active=g_enabled && g_fresh[d.side];
            auto target=active?Map(d.base[r],d.side==1,r==0,c,f):d.base[r];
            auto smoothed=active && d.blended[r]?Blend(d.output[r],target,1.f-std::exp(-dt/0.035f)):target;
            // Smoothing must not delay the wrist/root or unrelated fingers from the native animation.
            d.output[r]=target;
            if(active) {
                for(int i=6;i<=10;++i)d.output[r][i]=smoothed[i];
                d.output[r][27]=smoothed[27];
                if(f.contact){for(int i=2;i<=5;++i)d.output[r][i]=smoothed[i];d.output[r][26]=smoothed[26];}
            }
            if(active){Auxiliaries(d.output[r],true);if(!f.contact)d.output[r][26]=d.base[r][26];}
            d.blended[r]=active;
            writes.push_back({entry.first,d.handle,r,d.output[r]});
        }
    }
    }
    // A native input callback can hold SteamVR's lock and wait for g_mutex.
    // Publishing while holding g_mutex would invert those locks and trip the server watchdog.
    publish(writes,input);
}

bool Debug(PropertyContainerHandle_t container,char* response,uint32_t size){
    std::lock_guard<std::mutex> lock(g_mutex);auto it=g_devices.find(container);
    if(it==g_devices.end() || it->second.side<0 || !response || !size)return false;
    const auto& d=it->second;const auto q=d.output[1][7].orientation;
    std::snprintf(response,size,"mapping=%s own=%d frames=%llu native=%llu enabled=%d valid=%d age=%.1f proximity=%.4f slide=%.4f index_q=%.6f/%.6f/%.6f/%.6f",
        MappingRevision,int(d.own),static_cast<unsigned long long>(d.frames),static_cast<unsigned long long>(d.native_updates),int(g_enabled),int(d.finger.valid),d.finger.age_ms,d.finger.proximity,d.finger.slide,q.w,q.x,q.y,q.z);
    return true;
}
std::string Report() {
    std::lock_guard<std::mutex> lock(g_mutex);std::string result;
    for(int side=0;side<2;++side){const char* status=g_input?"waiting":"unavailable";
        for(auto& entry:g_devices){auto& d=entry.second;if(d.side!=side)continue;
            status=d.error?"error":!g_enabled?"disabled":(!g_fresh[side] || !d.have[0] || !d.have[1])?"waiting":"active";
        }result+=std::string(side?" skelright=":" skelleft=")+status;
    }return result;
}
}
