<script setup lang="ts">
import { ref, watch } from 'vue'
import { translate, type Language, type TranslationKey } from '@/i18n'

type InputMode = 'touchpad' | 'button' | 'two_buttons'
const props=defineProps<{mode:InputMode;pressThreshold:number;releaseThreshold:number;hapticAmplitude:number;busy:boolean;language:Language}>()
const t=(key:TranslationKey)=>translate(props.language,key)
const emit=defineEmits<{select:[mode:InputMode];thresholds:[press:number,release:number];amplitude:[value:number]}>()
const draftPress=ref(props.pressThreshold)
const draftRelease=ref(props.releaseThreshold)
const draftAmplitude=ref(props.hapticAmplitude)
watch(()=>props.pressThreshold,value=>draftPress.value=value)
watch(()=>props.releaseThreshold,value=>draftRelease.value=value)
watch(()=>props.hapticAmplitude,value=>draftAmplitude.value=value)
function onThresholdInput(which:'press'|'release',event:Event){
  const value=Number((event.target as HTMLInputElement).value)
  if(which==='press'){
    draftPress.value=value
    if(draftRelease.value>value)draftRelease.value=value
  }else{
    draftRelease.value=value
    if(draftPress.value<value)draftPress.value=value
  }
}
function saveThresholds(){emit('thresholds',draftPress.value,draftRelease.value)}
function onAmplitudeInput(event:Event){draftAmplitude.value=Number((event.target as HTMLInputElement).value)}
function saveAmplitude(){emit('amplitude',draftAmplitude.value)}
</script>
<template>
  <div class="mode-panel">
    <div class="input-modes" role="group" :aria-label="t('modeGroup')">
      <button type="button" :class="{active:mode==='touchpad'}" :aria-pressed="mode==='touchpad'" :disabled="busy" @click="emit('select','touchpad')"><b>{{t('touchpad')}}</b><small>{{t('position')}}</small></button>
      <button type="button" :class="{active:mode==='button'}" :aria-pressed="mode==='button'" :disabled="busy" @click="emit('select','button')"><b>{{t('button')}}</b><small>{{t('wholePad')}}</small></button>
      <button type="button" :class="{active:mode==='two_buttons'}" :aria-pressed="mode==='two_buttons'" :disabled="busy" @click="emit('select','two_buttons')"><b>{{t('twoButtons')}}</b><small>{{t('upperLower')}}</small></button>
    </div>
    <p v-if="mode==='touchpad'" class="mode-note">{{t('touchpadNote')}}</p>
    <template v-else>
      <div class="threshold-control">
        <div class="threshold-values">
          <label for="release-threshold">{{t('releaseAt')}} <b>{{draftRelease.toFixed(2)}}</b></label>
          <label for="press-threshold">{{t('pressStrength')}} <b>{{draftPress.toFixed(2)}}</b></label>
        </div>
        <div class="dual-range" :style="{'--release-pct':`${draftRelease*100}%`,'--press-pct':`${draftPress*100}%`}">
          <input id="release-threshold" :aria-label="t('releaseAt')" type="range" min="0" max="1" step="0.01" :value="draftRelease" :disabled="busy" @input="onThresholdInput('release',$event)" @change="saveThresholds" />
          <input id="press-threshold" :aria-label="t('pressStrength')" type="range" min="0" max="1" step="0.01" :value="draftPress" :disabled="busy" @input="onThresholdInput('press',$event)" @change="saveThresholds" />
        </div>
      </div>
      <div class="threshold-control">
        <label for="haptic-amplitude">{{t('hapticPower')}} <b>{{Math.round(draftAmplitude*100)}}%</b></label>
        <input id="haptic-amplitude" type="range" min="0" max="1" step="0.01" :value="draftAmplitude" :disabled="busy" @input="onAmplitudeInput" @change="saveAmplitude" />
      </div>
    </template>
  </div>
</template>
