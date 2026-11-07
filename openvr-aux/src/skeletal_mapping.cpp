#include "skeletal_mapping.hpp"
#include "hand_simulation.h"
#include <algorithm>
#include <cmath>
namespace qptp::skeleton {
namespace {
float clamp(float v, float lo = 0, float hi = 1) { return std::isfinite(v) ? std::clamp(v,lo,hi) : lo; }
vr::HmdQuaternionf_t unit(vr::HmdQuaternionf_t q) {
    const float n=std::sqrt(q.w*q.w+q.x*q.x+q.y*q.y+q.z*q.z);
    if (!(n>0) || !std::isfinite(n)) return {1,0,0,0};
    return {q.w/n,q.x/n,q.y/n,q.z/n};
}
vr::HmdQuaternionf_t mul(vr::HmdQuaternionf_t a, vr::HmdQuaternionf_t b) {
    return {a.w*b.w-a.x*b.x-a.y*b.y-a.z*b.z,a.w*b.x+a.x*b.w+a.y*b.z-a.z*b.y,
        a.w*b.y-a.x*b.z+a.y*b.w+a.z*b.x,a.w*b.z+a.x*b.y-a.y*b.x+a.z*b.w};
}
vr::VRBoneTransform_t compose(const vr::VRBoneTransform_t& a,const vr::VRBoneTransform_t& b) {
    vr::VRBoneTransform_t o{}; auto q=unit(a.orientation);
    auto v=mul(mul(q,{0,b.position.v[0],b.position.v[1],b.position.v[2]}),{q.w,-q.x,-q.y,-q.z});
    o.position={{a.position.v[0]+v.x,a.position.v[1]+v.y,a.position.v[2]+v.z,1}};
    o.orientation=unit(mul(q,b.orientation));return o;
}
struct Vec {float x,y,z;};
Vec operator+(Vec a,Vec b){return {a.x+b.x,a.y+b.y,a.z+b.z};}
Vec operator-(Vec a,Vec b){return {a.x-b.x,a.y-b.y,a.z-b.z};}
Vec operator*(Vec a,float s){return {a.x*s,a.y*s,a.z*s};}
float dot(Vec a,Vec b){return a.x*b.x+a.y*b.y+a.z*b.z;}
Vec cross(Vec a,Vec b){return {a.y*b.z-a.z*b.y,a.z*b.x-a.x*b.z,a.x*b.y-a.y*b.x};}
Vec normalized(Vec a,Vec fallback={1,0,0}){float n=std::sqrt(dot(a,a));return n>1e-7f?a*(1/n):fallback;}
Vec position(const vr::VRBoneTransform_t& t){return {t.position.v[0],t.position.v[1],t.position.v[2]};}
vr::HmdQuaternionf_t rotation(Vec axis,float radians){axis=normalized(axis);float s=std::sin(radians*.5f);return {std::cos(radians*.5f),axis.x*s,axis.y*s,axis.z*s};}
struct Palm {Vec across,forward,normal;};
Palm palm(const Pose& p,bool right){
    // Metacarpal translations are in wrist space, independent of the finger's local axes.
    Vec across=normalized(position(p[6])-position(p[21]),{0,1,0});
    if(right)across=across*-1.f;
    Vec forward=position(p[11]);forward=normalized(forward-across*dot(forward,across),{0,0,-1});
    return {across,forward,normalized(cross(forward,across))};
}
Pose simulate(bool right, MyFingerCurls curls, MyFingerSplays splays) {
    Pose p{};MyHandSimulation sim;
    sim.ComputeSkeletonTransforms(right?vr::TrackedControllerRole_RightHand:vr::TrackedControllerRole_LeftHand,curls,splays,p.data());
    for(auto& b:p) b.orientation=unit(b.orientation);
    Auxiliaries(p);return p;
}
}
void Auxiliaries(Pose& p,bool thumb_only) {
    // Runtime GetBoneHierarchy reports parent 0 (root) for auxiliary bones.
    for(int f=0;f<(thumb_only?2:5);++f) {
        const int first=f==0?2:6+(f-1)*5, last=f==0?5:first+4;
        auto tip=p[first];for(int i=first+1;i<=last;++i) tip=compose(tip,p[i]);
        p[26+f]=compose(p[1],tip);
    }
}
Pose Base(bool right,bool with_controller,const Controls& c) {
    const float grip=clamp(c.grip)*(with_controller?0.65f:1.f);
    const float index=(std::max)(clamp(c.trigger),c.trigger_touch?0.15f:0.f)*(with_controller?0.65f:1.f);
    return simulate(right,{c.thumb_touch?0.35f:0.f,index,grip,grip,grip},{});
}
Pose Blend(const Pose& a,const Pose& b,float weight) {
    Pose p{};const float t=clamp(weight);
    for(size_t i=0;i<BoneCount;++i) {
        for(int j=0;j<4;++j)p[i].position.v[j]=a[i].position.v[j]+t*(b[i].position.v[j]-a[i].position.v[j]);
        auto x=unit(a[i].orientation),y=unit(b[i].orientation);
        if(x.w*y.w+x.x*y.x+x.y*y.y+x.z*y.z<0)y={-y.w,-y.x,-y.y,-y.z};
        p[i].orientation=unit({x.w+t*(y.w-x.w),x.x+t*(y.x-x.x),x.y+t*(y.y-x.y),x.z+t*(y.z-x.z)});
    }return p;
}
Pose Map(const Pose& base,bool right,bool with_controller,const Controls& c,const FingerState& f) {
    if(!f.valid || f.age_ms>=500 || !std::isfinite(f.proximity) || !std::isfinite(f.slide))return base;
    Pose out=base;
    // Hardware-normalized channels; polarity and physical fit must be verified on Quest Pro.
    // Measured proximity saturates at 1 already when resting on an unpressed trigger.
    // Reserve the remaining curl for the ordinary trigger so pressing still animates the finger.
    const float trigger=clamp(c.trigger);
    float curl=trigger+(1.f-trigger)*0.35f*clamp(f.proximity);
    if(with_controller)curl*=0.65f;
    auto index=simulate(right,{0,curl,0,0,0},{});
    for(int i=7;i<=10;++i)out[i].orientation=index[i].orientation;
    auto axes=palm(base,right);
    // Slide adds flexion at the first phalanx using the exact same sample curl axis
    // as trigger input. Leave the distal joints' trigger/proximity curl unchanged.
    // Slide may read zero with a lifted finger. It is positional data only while
    // touching/pressing the trigger; never interpret no contact as an endpoint.
    const float slide=(c.trigger_touch || trigger>.01f)?1-2*clamp(f.slide):0.f;
    const float slide_curl=slide*(slide<0?1.f/3.f:1.f/6.f); // 30 degrees out, 15 in
    auto steered=simulate(right,{0,curl+slide_curl,0,0,0},{});
    out[7].orientation=steered[7].orientation;
    if(f.contact) {
        // X rotates the entire thumb chain at its CMC/base joint. Y curls the
        // descendants in that dynamically rotated plane; no fingertip IK/translation.
        const float x=clamp(f.x,-1,1),y=clamp(f.y,-1,1);
        auto thumb=simulate(right,{.35f-.18f*y,0,0,0,0},{});
        for(int i=2;i<=5;++i)out[i].orientation=thumb[i].orientation;
        const float angle=-x*0.6981317f*(right?-1.f:1.f); // mirrored +/-40 degrees, pad X inverted
        // Abduct out of the palm plane, rather than sweeping alongside the fingers.
        out[2].orientation=unit(mul(rotation(axes.across,angle),thumb[2].orientation));
    }
    Auxiliaries(out,true);if(!f.contact)out[26]=base[26];return out;
}
}
