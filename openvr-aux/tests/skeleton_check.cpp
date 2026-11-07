// Read-only SteamVR client used for hardware verification. Does not register drivers.
#include <openvr.h>
#include <windows.h>
#include <chrono>
#include <cstdio>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <thread>
int wmain(int argc,wchar_t** argv) {
    if(argc<2){std::cerr<<"Usage: skeleton_check.exe <SteamVR/bin/win64/openvr_api.dll> [seconds=60] [output.csv]\n";return 2;}
    HMODULE dll=LoadLibraryW(argv[1]);if(!dll){std::cerr<<"Cannot load openvr_api.dll\n";return 2;}
    auto init=reinterpret_cast<uint32_t(*)(vr::EVRInitError*,vr::EVRApplicationType,const char*)>(GetProcAddress(dll,"VR_InitInternal2"));
    auto shutdown=reinterpret_cast<void(*)()>(GetProcAddress(dll,"VR_ShutdownInternal"));
    auto get=reinterpret_cast<void*(*)(const char*,vr::EVRInitError*)>(GetProcAddress(dll,"VR_GetGenericInterface"));
    if(!init || !shutdown || !get){std::cerr<<"Missing OpenVR exports\n";FreeLibrary(dll);return 2;}
    vr::EVRInitError error;init(&error,vr::VRApplication_Background,nullptr);
    if(error!=vr::VRInitError_None){std::cerr<<"OpenVR init failed: "<<int(error)<<". Start SteamVR first.\n";FreeLibrary(dll);return 1;}
    auto* debug=static_cast<vr::IVRDebug*>(get(vr::IVRDebug_Version,&error));
    auto* system=static_cast<vr::IVRSystem*>(get(vr::IVRSystem_Version,&error));
    auto* input=static_cast<vr::IVRInput*>(get(vr::IVRInput_Version,&error));
    if(!input || error!=vr::VRInitError_None){std::cerr<<"IVRInput unavailable\n";shutdown();FreeLibrary(dll);return 1;}
    wchar_t exe[32768];GetModuleFileNameW(nullptr,exe,32768);
    auto manifest=(std::filesystem::path(exe).parent_path()/(argc>4?argv[4]:L"skeleton_check_actions.json")).u8string();
    if(input->SetActionManifestPath(manifest.c_str())!=vr::VRInputError_None){std::cerr<<"Manifest failed\n";shutdown();FreeLibrary(dll);return 1;}
    const bool vrchat=argc>5 && std::wcscmp(argv[5],L"vrchat")==0;
    vr::VRActionSetHandle_t set;vr::VRActionHandle_t actions[2];
    if(input->GetActionSetHandle(vrchat?"/actions/global":"/actions/check",&set)!=vr::VRInputError_None ||
       input->GetActionHandle(vrchat?"/actions/global/in/skeletonlefthand":"/actions/check/in/left",&actions[0])!=vr::VRInputError_None ||
       input->GetActionHandle(vrchat?"/actions/global/in/skeletonrighthand":"/actions/check/in/right",&actions[1])!=vr::VRInputError_None){std::cerr<<"Action handles failed\n";shutdown();FreeLibrary(dll);return 1;}
    std::ofstream csv;if(argc>3)csv.open(std::filesystem::path(argv[3]));
    if(argc>3 && !csv){std::cerr<<"Cannot open output CSV\n";shutdown();FreeLibrary(dll);return 2;}
    if(csv)csv<<"time_ms,side,active,thumb_curl,index_curl,bones_with_error,bones_without_error,device_index,origin,wrist_x,wrist_y,wrist_z,wrist_w,wrist_qx,wrist_qy,wrist_qz,vd_left_connected,vd_right_connected,vd_left_valid,vd_right_valid\n";
    const int seconds=argc>2?_wtoi(argv[2]):60;auto start=std::chrono::steady_clock::now();int tick=0;bool seen[2]{};
    while(std::chrono::steady_clock::now()-start<std::chrono::seconds(seconds)) {
        vr::VRActiveActionSet_t active{};active.ulActionSet=set;
        auto e=input->UpdateActionState(&active,sizeof(active),1);
        if(e!=vr::VRInputError_None){std::cerr<<"UpdateActionState failed: "<<int(e)<<"\n";break;}
        vr::TrackedDevicePose_t poses[16]{};if(system)system->GetDeviceToAbsoluteTrackingPose(vr::TrackingUniverseStanding,0,poses,16);
        if(tick==0 && system){
            for(unsigned i=0;i<16;++i){char type[128]{};system->GetStringTrackedDeviceProperty(i,vr::Prop_ControllerType_String,type,sizeof(type));
                if(type[0])std::cout<<"device "<<i<<" type="<<type<<" connected="<<poses[i].bDeviceIsConnected<<" valid="<<poses[i].bPoseIsValid<<'\n';
                if(debug && poses[i].bDeviceIsConnected){char details[1024]{};debug->DriverDebugRequest(i,"qptp_skeleton",details,sizeof(details));if(details[0])std::cout<<"device "<<i<<" driver="<<details<<'\n';}}}
        for(int hand=0;hand<2;++hand) {
            if(tick==0){vr::BoneIndex_t parents[31]{};input->GetBoneHierarchy(actions[hand],parents,31);std::cout<<"aux parents="<<parents[26]<<'/'<<parents[27]<<'\n';vr::VRInputValueHandle_t origins[16]{};input->GetActionOrigins(set,actions[hand],origins,16);
                for(auto handle:origins)if(handle){vr::InputOriginInfo_t oi{};input->GetOriginTrackedDeviceInfo(handle,&oi,sizeof(oi));std::cout<<"binding hand="<<hand<<" device="<<oi.trackedDeviceIndex<<'\n';}}
            vr::InputSkeletalActionData_t data{};vr::VRSkeletalSummaryData_t summary{};vr::VRBoneTransform_t bones[31]{};
            auto action_error=input->GetSkeletalActionData(actions[hand],&data,sizeof(data));
            vr::EVRSkeletalTrackingLevel level=vr::VRSkeletalTracking_Estimated;input->GetSkeletalTrackingLevel(actions[hand],&level);
            vr::InputOriginInfo_t origin{};
            input->GetOriginTrackedDeviceInfo(data.activeOrigin,&origin,sizeof(origin));
            auto summary_error=input->GetSkeletalSummaryData(actions[hand],vr::VRSummaryType_FromAnimation,&summary);
            auto with=input->GetSkeletalBoneData(actions[hand],vr::VRSkeletalTransformSpace_Parent,vr::VRSkeletalMotionRange_WithController,bones,31);
            auto without=input->GetSkeletalBoneData(actions[hand],vr::VRSkeletalTransformSpace_Parent,vr::VRSkeletalMotionRange_WithoutController,bones,31);
            bool ok=action_error==vr::VRInputError_None && summary_error==vr::VRInputError_None && data.bActive;
            seen[hand]|=ok && with==vr::VRInputError_None && without==vr::VRInputError_None;
            auto ms=std::chrono::duration_cast<std::chrono::milliseconds>(std::chrono::steady_clock::now()-start).count();
            if(csv)csv<<ms<<','<<(hand?"right":"left")<<','<<ok<<','<<summary.flFingerCurl[0]<<','<<summary.flFingerCurl[1]<<','<<int(with)<<','<<int(without)<<','<<origin.trackedDeviceIndex<<','<<data.activeOrigin<<','<<bones[1].position.v[0]<<','<<bones[1].position.v[1]<<','<<bones[1].position.v[2]<<','<<bones[1].orientation.w<<','<<bones[1].orientation.x<<','<<bones[1].orientation.y<<','<<bones[1].orientation.z<<','<<poses[3].bDeviceIsConnected<<','<<poses[4].bDeviceIsConnected<<','<<poses[3].bPoseIsValid<<','<<poses[4].bPoseIsValid<<'\n';
            if(tick%200==0 && debug && data.bActive){char details[1024]{};debug->DriverDebugRequest(origin.trackedDeviceIndex,"qptp_skeleton",details,sizeof(details));
                auto q=bones[7].orientation;std::cout<<"received index_q="<<q.w<<'/'<<q.x<<'/'<<q.y<<'/'<<q.z<<" driver="<<details<<'\n';}
            if(tick%200==0)std::cout<<(hand?"right":"left")<<" active="<<ok<<" level="<<int(level)<<" thumb="<<summary.flFingerCurl[0]<<" index="<<summary.flFingerCurl[1]<<" device="<<origin.trackedDeviceIndex<<" origin="<<data.activeOrigin<<" bones="<<int(with)<<'/'<<int(without)<<'\n';
        }
        ++tick;std::this_thread::sleep_for(std::chrono::milliseconds(5));
    }
    shutdown();FreeLibrary(dll);
    std::cout<<"Both hands returned active skeletal data: "<<(seen[0]&&seen[1]?"yes":"no")<<'\n';return seen[0]&&seen[1]?0:1;
}
