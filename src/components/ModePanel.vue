<script setup lang="ts">
import { ref, watch } from 'vue'
import { translate, type Language, type TranslationKey } from '@/i18n'

type InputMode = 'touchpad' | 'button' | 'two_buttons'
const props=defineProps<{mode:InputMode;pressThreshold:number;hapticAmplitude:number;busy:boolean;language:Language}>()
const t=(key:TranslationKey)=>translate(props.language,key)
const emit=defineEmits<{select:[mode:InputMode];threshold:[value:number];amplitude:[value:number]}>()
const draftThreshold=ref(props.pressThreshold)
const draftAmplitude=ref(props.hapticAmplitude)
watch(()=>props.pressThreshold,value=>draftThreshold.value=value)
watch(()=>props.hapticAmplitude,value=>draftAmplitude.value=value)
function onSliderInput(event:Event){draftThreshold.value=Number((event.target as HTMLInputElement).value)}
function saveThreshold(){emit('threshold',draftThreshold.value)}
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
        <label for="press-threshold">{{t('pressStrength')}} <b>{{draftThreshold.toFixed(2)}}</b></label>
        <input id="press-threshold" type="range" min="0.21" max="1" step="0.01" :value="draftThreshold" :disabled="busy" @input="onSliderInput" @change="saveThreshold" />
        <small>{{t('releaseAt')}}</small>
      </div>
      <div class="threshold-control">
        <label for="haptic-amplitude">{{t('hapticPower')}} <b>{{Math.round(draftAmplitude*100)}}%</b></label>
        <input id="haptic-amplitude" type="range" min="0" max="1" step="0.01" :value="draftAmplitude" :disabled="busy" @input="onAmplitudeInput" @change="saveAmplitude" />
      </div>
    </template>
  </div>
</template>
