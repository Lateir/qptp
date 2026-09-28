<script setup lang="ts">
import { onMounted, onUnmounted, reactive, watch } from 'vue'
import { translate, type Language, type TranslationKey } from '@/i18n'

type SensorValues = { stylus:number; triggerProximity:number; triggerSlide:number }
const props=defineProps<{left:SensorValues;right:SensorValues;leftOnline:boolean;rightOnline:boolean;language:Language}>()
const t=(key:TranslationKey)=>translate(props.language,key)
const verticalSensors=[{key:'stylus',label:'stylusSensor'},{key:'triggerProximity',label:'triggerProximitySensor'}] as const
function reading(value:number,online:boolean){return online&&Number.isFinite(value)?value.toFixed(3):'—'}
function level(value:number){return `${Math.max(0,Math.min(1,Number.isFinite(value)?value:0))*100}%`}

const slide=reactive({left:0,right:0})
const target={left:0,right:0}
let frame=0
let previous=0
watch(()=>[props.left.triggerSlide,props.leftOnline,props.right.triggerSlide,props.rightOnline] as const,([left,leftOnline,right,rightOnline])=>{
  target.left=leftOnline&&Number.isFinite(left)?Math.max(0,Math.min(1,left)):0
  target.right=rightOnline&&Number.isFinite(right)?Math.max(0,Math.min(1,right)):0
  if(target.left===0)slide.left=0
  if(target.right===0)slide.right=0
},{immediate:true})
function animate(time:number){
  const elapsed=previous?Math.min(time-previous,100):16
  const weight=1-Math.exp(-elapsed/50)
  slide.left+=((target.left-slide.left)*weight)
  slide.right+=((target.right-slide.right)*weight)
  previous=time
  frame=requestAnimationFrame(animate)
}
onMounted(()=>{frame=requestAnimationFrame(animate)})
onUnmounted(()=>cancelAnimationFrame(frame))
function slidePosition(side:'left'|'right'){return `${(side==='left'?1-slide.left:slide.right)*100}%`}
function slidePresent(side:'left'|'right'){return side==='left'?target.left>0:target.right>0}
</script>
<template>
  <div class="sensors-panel">
    <div class="sensor-cards">
      <section v-for="sensor in verticalSensors" :key="sensor.key" class="sensor-card">
        <h2>{{t(sensor.label)}}</h2>
        <div class="sensor-readings sensor-vertical-readings">
          <div v-for="side in (['left','right'] as const)" :key="side" :class="{offline:side==='left'?!leftOnline:!rightOnline}">
            <div class="sensor-bar" aria-hidden="true"><span :style="{height:level(side==='left'?left[sensor.key]:right[sensor.key])}"></span></div>
            <output :aria-label="t(side)">{{reading(side==='left'?left[sensor.key]:right[sensor.key],side==='left'?leftOnline:rightOnline)}}</output>
          </div>
        </div>
      </section>
      <section class="sensor-card sensor-slide-card">
        <h2>{{t('triggerSlideSensor')}}</h2>
        <div class="sensor-readings sensor-slide-readings">
          <div v-for="side in (['left','right'] as const)" :key="side" :class="{offline:side==='left'?!leftOnline:!rightOnline}">
            <div class="sensor-slide-track" aria-hidden="true">
              <span v-if="slidePresent(side)" class="sensor-slide-thumb" :style="{left:slidePosition(side)}"></span>
            </div>
            <output :aria-label="t(side)">{{reading(slide[side],side==='left'?leftOnline:rightOnline)}}</output>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>
