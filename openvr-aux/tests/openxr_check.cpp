// Manual, zero-layer D3D11 session for verifying SteamVR's OpenXR hand-joint bridge.
// Run separately from VRChat: it temporarily becomes the active OpenXR scene application.
#define XR_USE_PLATFORM_WIN32
#define XR_USE_GRAPHICS_API_D3D11
#define XR_NO_PROTOTYPES
#include <windows.h>
#include <d3d11.h>
#include <dxgi1_2.h>
#include <wrl/client.h>
#include <openxr/openxr.h>
#include <openxr/openxr_platform.h>
#include <chrono>
#include <cstring>
#include <functional>
#include <iostream>
#include <stdexcept>
#include <string>
#include <thread>
#include <vector>
using Microsoft::WRL::ComPtr;
struct Cleanup { std::function<void()> fn; ~Cleanup(){fn();} };
void check(XrResult r,const char* action){if(XR_FAILED(r))throw std::runtime_error(std::string(action)+" result="+std::to_string(r));}
int wmain(int argc,wchar_t** argv){
    if(argc<2){std::cerr<<"Usage: openxr_check.exe <openxr_loader.dll> [seconds=15]\nUses the currently selected OpenXR runtime. Close other VR scene apps first.\n";return 2;}
    HMODULE module=LoadLibraryW(argv[1]);if(!module){std::cerr<<"Cannot load OpenXR loader\n";return 2;}
    Cleanup library{[&]{FreeLibrary(module);}};
    auto get=reinterpret_cast<PFN_xrGetInstanceProcAddr>(GetProcAddress(module,"xrGetInstanceProcAddr"));
    if(!get){std::cerr<<"xrGetInstanceProcAddr missing\n";return 2;}
    try {
        XrInstance instance=XR_NULL_HANDLE;
#define LOAD(name) PFN_##name name=nullptr;check(get(instance,#name,reinterpret_cast<PFN_xrVoidFunction*>(&name)),#name)
        LOAD(xrEnumerateInstanceExtensionProperties);LOAD(xrCreateInstance);
        uint32_t count=0;check(xrEnumerateInstanceExtensionProperties(nullptr,0,&count,nullptr),"extensions count");
        std::vector<XrExtensionProperties> extensions(count,{XR_TYPE_EXTENSION_PROPERTIES});
        check(xrEnumerateInstanceExtensionProperties(nullptr,count,&count,extensions.data()),"extensions");
        bool hands=false,d3d=false;
        for(auto& e:extensions){hands|=std::strcmp(e.extensionName,XR_EXT_HAND_TRACKING_EXTENSION_NAME)==0;d3d|=std::strcmp(e.extensionName,XR_KHR_D3D11_ENABLE_EXTENSION_NAME)==0;}
        std::cout<<"XR_EXT_hand_tracking="<<hands<<" D3D11="<<d3d<<'\n';
        if(!hands || !d3d)return 2;
        const char* enabled[]={XR_EXT_HAND_TRACKING_EXTENSION_NAME,XR_KHR_D3D11_ENABLE_EXTENSION_NAME};
        XrInstanceCreateInfo info{XR_TYPE_INSTANCE_CREATE_INFO};std::strcpy(info.applicationInfo.applicationName,"qptp joint verification");
        info.applicationInfo.apiVersion=XR_MAKE_VERSION(1,0,0);info.enabledExtensionCount=2;info.enabledExtensionNames=enabled;
        check(xrCreateInstance(&info,&instance),"create instance");
        LOAD(xrDestroyInstance);Cleanup instance_cleanup{[&]{xrDestroyInstance(instance);}};
        LOAD(xrGetInstanceProperties);XrInstanceProperties ip{XR_TYPE_INSTANCE_PROPERTIES};check(xrGetInstanceProperties(instance,&ip),"runtime properties");
        std::cout<<"Runtime="<<ip.runtimeName<<" version="<<ip.runtimeVersion<<'\n';
        LOAD(xrGetSystem);LOAD(xrGetSystemProperties);LOAD(xrGetD3D11GraphicsRequirementsKHR);
        XrSystemGetInfo system_info{XR_TYPE_SYSTEM_GET_INFO};system_info.formFactor=XR_FORM_FACTOR_HEAD_MOUNTED_DISPLAY;
        XrSystemId system;check(xrGetSystem(instance,&system_info,&system),"get system");
        XrSystemHandTrackingPropertiesEXT hp{XR_TYPE_SYSTEM_HAND_TRACKING_PROPERTIES_EXT};XrSystemProperties sp{XR_TYPE_SYSTEM_PROPERTIES};sp.next=&hp;
        check(xrGetSystemProperties(instance,system,&sp),"system properties");
        std::cout<<"supportsHandTracking="<<hp.supportsHandTracking<<'\n';if(!hp.supportsHandTracking)return 2;
        if(argc>2 && std::wcscmp(argv[2],L"--capabilities")==0)return 0;
        XrGraphicsRequirementsD3D11KHR req{XR_TYPE_GRAPHICS_REQUIREMENTS_D3D11_KHR};check(xrGetD3D11GraphicsRequirementsKHR(instance,system,&req),"graphics requirements");
        ComPtr<IDXGIFactory1> factory;if(FAILED(CreateDXGIFactory1(IID_PPV_ARGS(&factory))))throw std::runtime_error("DXGI factory failed");
        ComPtr<IDXGIAdapter1> adapter;
        for(UINT i=0;;++i){ComPtr<IDXGIAdapter1> candidate;if(factory->EnumAdapters1(i,&candidate)==DXGI_ERROR_NOT_FOUND)break;
            DXGI_ADAPTER_DESC1 desc{};candidate->GetDesc1(&desc);if(std::memcmp(&desc.AdapterLuid,&req.adapterLuid,sizeof(LUID))==0){adapter=candidate;break;}}
        if(!adapter)throw std::runtime_error("OpenXR adapter not found");
        ComPtr<ID3D11Device> device;ComPtr<ID3D11DeviceContext> context;D3D_FEATURE_LEVEL actual;
        if(FAILED(D3D11CreateDevice(adapter.Get(),D3D_DRIVER_TYPE_UNKNOWN,nullptr,0,&req.minFeatureLevel,1,D3D11_SDK_VERSION,&device,&actual,&context)))throw std::runtime_error("D3D11 device failed");
        LOAD(xrCreateSession);LOAD(xrDestroySession);LOAD(xrCreateReferenceSpace);LOAD(xrDestroySpace);
        LOAD(xrCreateHandTrackerEXT);LOAD(xrDestroyHandTrackerEXT);LOAD(xrLocateHandJointsEXT);
        LOAD(xrPollEvent);LOAD(xrBeginSession);LOAD(xrEndSession);LOAD(xrWaitFrame);LOAD(xrBeginFrame);LOAD(xrEndFrame);
        LOAD(xrStringToPath);LOAD(xrGetCurrentInteractionProfile);LOAD(xrPathToString);
        XrGraphicsBindingD3D11KHR graphics{XR_TYPE_GRAPHICS_BINDING_D3D11_KHR};graphics.device=device.Get();
        XrSessionCreateInfo sci{XR_TYPE_SESSION_CREATE_INFO};sci.next=&graphics;sci.systemId=system;
        XrSession session;check(xrCreateSession(instance,&sci,&session),"create session");Cleanup session_cleanup{[&]{xrDestroySession(session);}};
        XrReferenceSpaceCreateInfo space_info{XR_TYPE_REFERENCE_SPACE_CREATE_INFO};space_info.referenceSpaceType=XR_REFERENCE_SPACE_TYPE_LOCAL;space_info.poseInReferenceSpace.orientation.w=1;
        XrSpace space;check(xrCreateReferenceSpace(session,&space_info,&space),"local space");Cleanup space_cleanup{[&]{xrDestroySpace(space);}};
        XrHandTrackerEXT trackers[2]{};Cleanup trackers_cleanup{[&]{for(auto t:trackers)if(t)xrDestroyHandTrackerEXT(t);}};
        XrPath paths[2];
        for(int h=0;h<2;++h){XrHandTrackerCreateInfoEXT ci{XR_TYPE_HAND_TRACKER_CREATE_INFO_EXT};ci.hand=h?XR_HAND_RIGHT_EXT:XR_HAND_LEFT_EXT;ci.handJointSet=XR_HAND_JOINT_SET_DEFAULT_EXT;
            check(xrCreateHandTrackerEXT(session,&ci,&trackers[h]),"create hand tracker");check(xrStringToPath(instance,h?"/user/hand/right":"/user/hand/left",&paths[h]),"hand path");}
        bool running=false,done=false,seen[2]{};int frames=0;auto start=std::chrono::steady_clock::now();
        const int seconds=argc>2?_wtoi(argv[2]):15;
        while(!done && std::chrono::steady_clock::now()-start<std::chrono::seconds(seconds)){
            XrEventDataBuffer event{XR_TYPE_EVENT_DATA_BUFFER};XrResult er;
            while((er=xrPollEvent(instance,&event))==XR_SUCCESS){
                if(event.type==XR_TYPE_EVENT_DATA_SESSION_STATE_CHANGED){auto& e=*reinterpret_cast<XrEventDataSessionStateChanged*>(&event);std::cout<<"Session state="<<e.state<<'\n';
                    if(e.state==XR_SESSION_STATE_READY){XrSessionBeginInfo bi{XR_TYPE_SESSION_BEGIN_INFO};bi.primaryViewConfigurationType=XR_VIEW_CONFIGURATION_TYPE_PRIMARY_STEREO;check(xrBeginSession(session,&bi),"begin session");running=true;}
                    if(e.state==XR_SESSION_STATE_STOPPING){check(xrEndSession(session),"end session");running=false;done=true;}
                    if(e.state==XR_SESSION_STATE_EXITING || e.state==XR_SESSION_STATE_LOSS_PENDING)done=true;
                }
                if(event.type==XR_TYPE_EVENT_DATA_INTERACTION_PROFILE_CHANGED)for(int h=0;h<2;++h){
                    XrInteractionProfileState profile{XR_TYPE_INTERACTION_PROFILE_STATE};check(xrGetCurrentInteractionProfile(session,paths[h],&profile),"interaction profile");
                    if(profile.interactionProfile){char name[XR_MAX_PATH_LENGTH];uint32_t length;check(xrPathToString(instance,profile.interactionProfile,sizeof(name),&length,name),"profile name");std::cout<<(h?"right":"left")<<" profile="<<name<<'\n';}}
                event={XR_TYPE_EVENT_DATA_BUFFER};
            }
            if(er!=XR_EVENT_UNAVAILABLE)check(er,"poll event");
            if(!running || done){std::this_thread::sleep_for(std::chrono::milliseconds(20));continue;}
            XrFrameWaitInfo wi{XR_TYPE_FRAME_WAIT_INFO};XrFrameState frame{XR_TYPE_FRAME_STATE};check(xrWaitFrame(session,&wi,&frame),"wait frame");
            XrFrameBeginInfo bi{XR_TYPE_FRAME_BEGIN_INFO};check(xrBeginFrame(session,&bi),"begin frame");
            for(int h=0;h<2;++h){XrHandJointLocationEXT joints[XR_HAND_JOINT_COUNT_EXT]{};
                XrHandJointLocationsEXT locations{XR_TYPE_HAND_JOINT_LOCATIONS_EXT};locations.jointCount=XR_HAND_JOINT_COUNT_EXT;locations.jointLocations=joints;
                XrHandJointsLocateInfoEXT li{XR_TYPE_HAND_JOINTS_LOCATE_INFO_EXT};li.baseSpace=space;li.time=frame.predictedDisplayTime;
                auto result=xrLocateHandJointsEXT(trackers[h],&li,&locations);int pos=0,rot=0,tracked=0;
                for(auto& j:joints){pos+=(j.locationFlags&XR_SPACE_LOCATION_POSITION_VALID_BIT)!=0;rot+=(j.locationFlags&XR_SPACE_LOCATION_ORIENTATION_VALID_BIT)!=0;tracked+=(j.locationFlags&XR_SPACE_LOCATION_POSITION_TRACKED_BIT)!=0;}
                seen[h]|=XR_SUCCEEDED(result) && locations.isActive && pos==XR_HAND_JOINT_COUNT_EXT && rot==XR_HAND_JOINT_COUNT_EXT;
                if(frames%90==0)std::cout<<(h?"right":"left")<<" result="<<result<<" active="<<locations.isActive<<" validPosition="<<pos<<" validOrientation="<<rot<<" tracked="<<tracked<<'\n';
            }
            XrFrameEndInfo ei{XR_TYPE_FRAME_END_INFO};ei.displayTime=frame.predictedDisplayTime;ei.environmentBlendMode=XR_ENVIRONMENT_BLEND_MODE_OPAQUE;
            check(xrEndFrame(session,&ei),"end frame");++frames;
        }
        std::cout<<"Both hands active with 26 valid joints: "<<(seen[0]&&seen[1]?"yes":"no")<<'\n';return seen[0]&&seen[1]?0:1;
    }catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}
}
