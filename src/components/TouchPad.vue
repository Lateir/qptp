<script setup lang="ts">
import { computed } from 'vue'
import { translate, type Language, type TranslationKey } from '@/i18n'
import type { PadOutput, TouchpadStyle } from '@/touchpad'
const props=defineProps<{x:number;y:number;force:number;live:boolean;mode:'touchpad'|'button'|'two_buttons';buttons:[boolean,boolean];threshold:number;releaseThreshold:number;pad:PadOutput;padStyle:TouchpadStyle;language:Language}>()
const t=(key:TranslationKey)=>translate(props.language,key)
const clamp=(v:number)=>Math.max(0,Math.min(1,v))
const nf=computed(()=>clamp(props.force))
// Button modes keep showing the raw force; the touchpad shows what SteamVR receives (Rust has already applied calibration and style).
const shown=computed(()=>props.live&&props.pad.contact!=='none')
const engaged=computed(()=>shown.value&&props.pad.contact==='active')
const mouse=computed(()=>props.padStyle==='mouse')
const readout=computed(()=>mouse.value?[props.pad.fingerX,props.pad.fingerY,nf.value]:[props.pad.x,props.pad.y,props.pad.force])
const fmt=(v:number)=>(v<-0.004?'−':'')+Math.abs(v).toFixed(2)
// Pad coordinates are -1..1 with +y up; the field is laid out top-down.
const at=(x:number,y:number)=>({left:`${(x+1)*50}%`,top:`${(1-y)*50}%`})
const finger=computed(()=>at(props.pad.fingerX,props.pad.fingerY))
const anchor=computed(()=>at(props.pad.anchorX,props.pad.anchorY))
// Dot grows from 12px (touch) to 48px (full press).
const size=computed(()=>props.pad.contact==='rejected'?24:12+clamp(mouse.value?props.force:props.pad.force)*36)
const glide=computed(()=>props.live&&props.padStyle==='swipe'&&(props.pad.x!==0||props.pad.y!==0))
const tip=computed(()=>at(props.pad.x*0.8,props.pad.y*0.8))
const active=computed(()=>props.live&&props.buttons.some(Boolean))
</script>
<template>
<section class="pad" :class="{live,'button-pad':mode!=='touchpad'}">
  <dl v-if="mode==='touchpad'" class="readout">
    <div><dt>X</dt><dd>{{fmt(readout[0])}}</dd></div>
    <div><dt>Y</dt><dd>{{fmt(readout[1])}}</dd></div>
    <div><dt>F</dt><dd>{{readout[2].toFixed(2)}}</dd></div>
  </dl>
  <template v-else>
    <div class="pad-force"><span>{{t('force')}}</span><b>{{nf.toFixed(2)}}</b></div>
    <div v-if="mode==='button'" class="pad-button" :class="{pressed:active}"><b>1</b></div>
    <div v-else class="pad-zones"><div :class="{pressed:live&&buttons[0]}"><b>1</b><span>{{t('upper')}}</span></div><div :class="{pressed:live&&buttons[1]}"><b>2</b><span>{{t('lower')}}</span></div></div>
    <div class="pad-threshold">{{releaseThreshold.toFixed(2)}} / {{threshold.toFixed(2)}}</div>
  </template>
  <div v-if="mode==='touchpad'" class="field">
    <svg class="trace" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
      <line v-if="padStyle==='joystick'&&engaged" :x1="(pad.anchorX+1)*50" :y1="(1-pad.anchorY)*50" :x2="(pad.fingerX+1)*50" :y2="(1-pad.fingerY)*50"/>
      <line v-if="glide" class="glide" x1="50" y1="50" :x2="(pad.x*0.8+1)*50" :y2="(1-pad.y*0.8)*50"/>
    </svg>
    <span v-if="padStyle==='joystick'&&engaged" class="anchor" :style="anchor"></span>
    <span v-if="glide" class="glide-tip" :style="tip"></span>
    <template v-if="shown">
      <span v-if="engaged&&pad.click" class="click-ring" :style="{...finger,width:size+12+'px',height:size+12+'px'}"></span>
      <svg v-if="mouse&&engaged" class="cursor-mark" :style="finger" viewBox="0 0 12 18" aria-hidden="true"><path d="M1 1v14l3.6-3.4 2.6 5.6 2.4-1.1-2.6-5.5H12z"/></svg>
      <span v-else class="dot" :class="pad.contact" :style="{...finger,width:size+'px',height:size+'px'}"></span>
    </template>
  </div>
</section>
</template>
