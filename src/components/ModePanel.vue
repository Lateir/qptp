<script setup lang="ts">
import { ref, watch } from 'vue'

type InputMode = 'touchpad' | 'button' | 'two_buttons'
const props=defineProps<{mode:InputMode;pressThreshold:number;hapticAmplitude:number;busy:boolean}>()
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
    <div class="input-modes" role="group" aria-label="Режим панели">
      <button type="button" :class="{active:mode==='touchpad'}" :aria-pressed="mode==='touchpad'" :disabled="busy" @click="emit('select','touchpad')"><b>Тачпад</b><small>позиция</small></button>
      <button type="button" :class="{active:mode==='button'}" :aria-pressed="mode==='button'" :disabled="busy" @click="emit('select','button')"><b>Кнопка</b><small>вся панель</small></button>
      <button type="button" :class="{active:mode==='two_buttons'}" :aria-pressed="mode==='two_buttons'" :disabled="busy" @click="emit('select','two_buttons')"><b>2 кнопки</b><small>верх / низ</small></button>
    </div>
    <p v-if="mode==='touchpad'" class="mode-note">Положение и усилие без виртуальных кнопок</p>
    <template v-else>
      <div class="threshold-control">
        <label for="press-threshold">Сила нажатия <b>{{draftThreshold.toFixed(2)}}</b></label>
        <input id="press-threshold" type="range" min="0.21" max="1" step="0.01" :value="draftThreshold" :disabled="busy" @input="onSliderInput" @change="saveThreshold" />
        <small>Отпускание при 0.20</small>
      </div>
      <div class="threshold-control">
        <label for="haptic-amplitude">Мощность вибрации <b>{{Math.round(draftAmplitude*100)}}%</b></label>
        <input id="haptic-amplitude" type="range" min="0" max="1" step="0.01" :value="draftAmplitude" :disabled="busy" @input="onAmplitudeInput" @change="saveAmplitude" />
      </div>
    </template>
  </div>
</template>
