<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { translate, type Language, type TranslationKey } from '@/i18n'
import { defaultCalibration, type Calibration } from '@/touchpad'

type Raw = { x:number; y:number; force:number; touch?:boolean|null }
const props=defineProps<{left:Raw;right:Raw;leftOnline:boolean;rightOnline:boolean;calibration:Calibration;busy:boolean;language:Language}>()
const emit=defineEmits<{save:[calibration:Calibration];cancel:[]}>()
const t=(key:TranslationKey)=>translate(props.language,key)
const sides=['left','right'] as const
type Side=typeof sides[number]
const steps:TranslationKey[]=['calibrationEdges','calibrationUp','calibrationForce']
const step=ref(0)
const draft=ref<Calibration>({...props.calibration})
// Raw coordinates from Rust: x as sent by the module, y already flipped so 0 is the top. Idle is (0, 255) unless the module reports a contact flag.
const touching=(side:Side)=>{const s=props[side];return (side==='left'?props.leftOnline:props.rightOnline)&&(s.touch??(s.x!==0||s.y!==255))}
const edges=reactive({xMin:Infinity,xMax:-Infinity,yMin:Infinity,yMax:-Infinity})
const strokes=reactive({left:{start:null as null|[number,number],last:[0,0] as [number,number],angle:null as null|number},right:{start:null as null|[number,number],last:[0,0] as [number,number],angle:null as null|number}})
const force=reactive({low:Infinity,high:-Infinity})
const edgesReady=computed(()=>edges.xMax-edges.xMin>=32&&edges.yMax-edges.yMin>=32)
const forceReady=computed(()=>force.high-force.low>=0.05)
function normalize([x,y]:[number,number]):[number,number]{const k=draft.value;return [(x-k.xMin)/(k.xMax-k.xMin)*2-1,1-(y-k.yMin)/(k.yMax-k.yMin)*2]}
// Angle of an upward stroke from straight up, clockwise positive; rotating the pad by it makes the stroke point up.
function finishStroke(side:Side){
  const stroke=strokes[side];if(!stroke.start)return
  const [x0,y0]=normalize(stroke.start),[x1,y1]=normalize(stroke.last);stroke.start=null
  const dx=x1-x0,dy=y1-y0
  if(Math.hypot(dx,dy)<0.5||dy<=0)return
  const angle=Math.atan2(dx,dy)*180/Math.PI
  if(Math.abs(angle)<=60)stroke.angle=Math.round(angle*2)/2
}
function sample(side:Side){
  const s=props[side],down=touching(side)
  if(step.value===0&&down){edges.xMin=Math.min(edges.xMin,s.x);edges.xMax=Math.max(edges.xMax,s.x);edges.yMin=Math.min(edges.yMin,s.y);edges.yMax=Math.max(edges.yMax,s.y)}
  if(step.value===1){const stroke=strokes[side];if(down){if(!stroke.start)stroke.start=[s.x,s.y];stroke.last=[s.x,s.y]}else finishStroke(side)}
  if(step.value===2&&down&&Number.isFinite(s.force)){force.low=Math.min(force.low,s.force);force.high=Math.max(force.high,s.force)}
}
watch(()=>props.left,()=>sample('left'))
watch(()=>props.right,()=>sample('right'))
function apply(){
  const k=draft.value
  if(step.value===0&&edgesReady.value)Object.assign(k,{xMin:edges.xMin,xMax:edges.xMax,yMin:edges.yMin,yMax:edges.yMax})
  if(step.value===1){if(strokes.left.angle!==null)k.rotationLeft=strokes.left.angle;if(strokes.right.angle!==null)k.rotationRight=strokes.right.angle}
  if(step.value===2&&forceReady.value)Object.assign(k,{forceLow:Math.round(Math.max(0,force.low)*100)/100,forceHigh:Math.round(Math.min(1,force.high)*100)/100})
}
function next(applyStep:boolean){if(applyStep)apply();if(step.value<2)step.value++;else emit('save',{...draft.value})}
function back(){if(step.value>0)step.value--}
const pct=(v:number)=>`${Math.max(0,Math.min(1,v))*100}%`
const box=computed(()=>edgesReady.value||edges.xMax>=edges.xMin?{left:pct(edges.xMin/255),top:pct(edges.yMin/255),width:pct((edges.xMax-edges.xMin)/255),height:pct((edges.yMax-edges.yMin)/255)}:null)
const degrees=(v:number)=>`${v>0?'+':v<0?'−':''}${Math.abs(v)}°`
function result(side:Side){
  if(step.value===0)return edgesReady.value?`${edges.xMin}–${edges.xMax} · ${edges.yMin}–${edges.yMax}`:'—'
  if(step.value===1){const angle=strokes[side].angle;return angle===null?degrees(side==='left'?draft.value.rotationLeft:draft.value.rotationRight):`${degrees(angle)} ✓`}
  return forceReady.value?`${force.low.toFixed(2)} – ${force.high.toFixed(2)}`:'—'
}
const ready=computed(()=>step.value===0?edgesReady.value:step.value===1?strokes.left.angle!==null||strokes.right.angle!==null:forceReady.value)
</script>
<template>
  <div class="calibration-panel">
    <header><h2>{{t('calibrationTitle')}}</h2><span>{{t('calibrationStep')}} {{step+1}}/3</span></header>
    <p class="calibration-hint">{{t(steps[step])}}</p>
    <div class="calibration-pads">
      <figure v-for="side in sides" :key="side" :class="{online:side==='left'?leftOnline:rightOnline}">
        <div class="mini-pad">
          <span v-if="step===0&&box" class="edge-box" :style="box"></span>
          <svg v-if="step===1&&strokes[side].start" class="trace" viewBox="0 0 255 255" preserveAspectRatio="none" aria-hidden="true"><line :x1="strokes[side].start![0]" :y1="strokes[side].start![1]" :x2="strokes[side].last[0]" :y2="strokes[side].last[1]"/></svg>
          <template v-if="step===2">
            <span class="force-band" :style="{bottom:pct(forceReady?force.low:0),height:pct(forceReady?force.high-force.low:0)}"></span>
            <span class="force-level" :style="{height:pct(touching(side)?props[side].force:0)}"></span>
          </template>
          <span v-else-if="touching(side)" class="mini-dot" :style="{left:pct(props[side].x/255),top:pct(props[side].y/255)}"></span>
        </div>
        <figcaption><b>{{t(side)}}</b><span>{{result(side)}}</span></figcaption>
      </figure>
    </div>
    <div class="calibration-actions">
      <button type="button" class="ghost-button" :disabled="busy" @click="emit('save',defaultCalibration())">{{t('reset')}}</button>
      <span></span>
      <button type="button" class="ghost-button" @click="step===0?emit('cancel'):back()">{{step===0?t('cancel'):t('back')}}</button>
      <button type="button" class="ghost-button" :disabled="busy" @click="next(false)">{{t('skip')}}</button>
      <button type="button" class="primary-button" :disabled="busy||!ready" @click="next(true)">{{step===2?t('save'):t('next')}}</button>
    </div>
  </div>
</template>
