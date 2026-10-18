<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { translate, type Language, type TranslationKey } from '@/i18n'
import type { TouchpadSettings, TouchpadStyle } from '@/touchpad'

type InputMode = 'touchpad' | 'button' | 'two_buttons'
const props=defineProps<{mode:InputMode;pressThreshold:number;releaseThreshold:number;hapticAmplitude:number;touchpad:TouchpadSettings;contactSize:boolean;busy:boolean;language:Language}>()
const t=(key:TranslationKey)=>translate(props.language,key)
const emit=defineEmits<{select:[mode:InputMode];thresholds:[press:number,release:number];amplitude:[value:number];touchpad:[settings:TouchpadSettings];calibrate:[]}>()
const draftPress=ref(props.pressThreshold)
const draftRelease=ref(props.releaseThreshold)
const draftAmplitude=ref(props.hapticAmplitude)
const draft=ref<TouchpadSettings>({...props.touchpad})
watch(()=>props.pressThreshold,value=>draftPress.value=value)
watch(()=>props.releaseThreshold,value=>draftRelease.value=value)
watch(()=>props.hapticAmplitude,value=>draftAmplitude.value=value)
watch(()=>props.touchpad,value=>draft.value={...value},{deep:true})
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
// Touchpad settings: sliders update the draft while dragging and save on release.
const styles:{id:TouchpadStyle;label:TranslationKey}[]=[{id:'absolute',label:'styleAbsolute'},{id:'joystick',label:'styleJoystick'},{id:'swipe',label:'styleSwipe'},{id:'mouse',label:'styleMouse'}]
const styleNote=computed(()=>t(({absolute:'styleAbsoluteNote',joystick:'styleJoystickNote',swipe:'styleSwipeNote',mouse:'styleMouseNote'} as const)[draft.value.style]))
function saveTouchpad(){emit('touchpad',{...draft.value})}
function selectStyle(style:TouchpadStyle){if(style===draft.value.style)return;draft.value.style=style;saveTouchpad()}
function onNumber(key:'joystickRange'|'swipeGain'|'mouseSpeed',event:Event){draft.value[key]=Number((event.target as HTMLInputElement).value)}
function onClickInput(which:'press'|'release',event:Event){
  const value=Number((event.target as HTMLInputElement).value)
  if(which==='press'){draft.value.clickPress=Math.max(value,0.05);if(draft.value.clickRelease>draft.value.clickPress)draft.value.clickRelease=draft.value.clickPress}
  else{draft.value.clickRelease=value;if(draft.value.clickPress<value)draft.value.clickPress=Math.max(value,0.05)}
}
function onPalm(event:Event){draft.value.palmRejection=(event.target as HTMLInputElement).checked;saveTouchpad()}
</script>
<template>
  <div class="mode-panel" :class="{compact:mode==='touchpad'}">
    <div class="input-modes" role="group" :aria-label="t('modeGroup')">
      <button type="button" :class="{active:mode==='touchpad'}" :aria-pressed="mode==='touchpad'" :disabled="busy" @click="emit('select','touchpad')"><b>{{t('touchpad')}}</b><small>X · Y · F</small></button>
      <button type="button" :class="{active:mode==='button'}" :aria-pressed="mode==='button'" :disabled="busy" @click="emit('select','button')"><b>{{t('button')}}</b><small>{{t('wholePad')}}</small></button>
      <button type="button" :class="{active:mode==='two_buttons'}" :aria-pressed="mode==='two_buttons'" :disabled="busy" @click="emit('select','two_buttons')"><b>{{t('twoButtons')}}</b><small>{{t('upperLower')}}</small></button>
    </div>
    <template v-if="mode==='touchpad'">
      <div class="style-picker">
        <div class="segmented" role="group" :aria-label="t('styleGroup')">
          <button v-for="item in styles" :key="item.id" type="button" :class="{active:draft.style===item.id}" :aria-pressed="draft.style===item.id" :disabled="busy" @click="selectStyle(item.id)">{{t(item.label)}}</button>
        </div>
        <p class="style-note">{{styleNote}}</p>
      </div>
      <div v-if="draft.style==='joystick'" class="threshold-control">
        <label for="joystick-range">{{t('joystickRange')}} <b>{{Math.round(draft.joystickRange*100)}}%</b></label>
        <input id="joystick-range" type="range" min="0.15" max="1" step="0.01" :value="draft.joystickRange" :disabled="busy" @input="onNumber('joystickRange',$event)" @change="saveTouchpad" />
      </div>
      <div v-else-if="draft.style==='swipe'" class="threshold-control">
        <label for="swipe-gain">{{t('swipeGain')}} <b>{{Math.round(draft.swipeGain*100)}}%</b></label>
        <input id="swipe-gain" type="range" min="0.05" max="1" step="0.01" :value="draft.swipeGain" :disabled="busy" @input="onNumber('swipeGain',$event)" @change="saveTouchpad" />
      </div>
      <div v-else-if="draft.style==='mouse'" class="threshold-control">
        <label for="mouse-speed">{{t('mouseSpeed')}} <b>{{draft.mouseSpeed}}</b></label>
        <input id="mouse-speed" type="range" min="200" max="4000" step="50" :value="draft.mouseSpeed" :disabled="busy" @input="onNumber('mouseSpeed',$event)" @change="saveTouchpad" />
      </div>
      <div class="threshold-control">
        <label>{{t('clickForce')}} <b>{{draft.clickRelease.toFixed(2)}} / {{draft.clickPress.toFixed(2)}}</b></label>
        <div class="dual-range" :style="{'--release-pct':`${draft.clickRelease*100}%`,'--press-pct':`${draft.clickPress*100}%`}">
          <input :aria-label="`${t('clickForce')}: ${t('releaseAt')}`" type="range" min="0" max="1" step="0.01" :value="draft.clickRelease" :disabled="busy" @input="onClickInput('release',$event)" @change="saveTouchpad" />
          <input :aria-label="`${t('clickForce')}: ${t('pressStrength')}`" type="range" min="0" max="1" step="0.01" :value="draft.clickPress" :disabled="busy" @input="onClickInput('press',$event)" @change="saveTouchpad" />
        </div>
      </div>
      <div class="pad-options">
        <label class="switch-row" :title="contactSize?'':t('palmNeedsModule')">
          <span>{{t('palmRejection')}}<small v-if="!contactSize">{{t('palmNeedsModule')}}</small></span>
          <input type="checkbox" role="switch" :checked="draft.palmRejection&&contactSize" :disabled="busy||!contactSize" @change="onPalm"/>
        </label>
        <button type="button" class="ghost-button" @click="emit('calibrate')">{{t('calibrate')}}</button>
      </div>
    </template>
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
