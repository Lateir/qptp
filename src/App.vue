<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { version } from '../package.json'
import ModePanel from '@/components/ModePanel.vue'
import TouchPad from '@/components/TouchPad.vue'
type Mode = 'usb' | 'lan'
type InputMode = 'touchpad' | 'button' | 'two_buttons'
type Side = { x:number; y:number; force:number }
type Snapshot = { phase:string; message:string; transport:Mode; endpoint?:string; protocol?:string; samples:number; left:Side; right:Side; status?:Record<string,unknown>; inputMode:InputMode; pressThreshold:number; hapticAmplitude:number; leftButtons:[boolean,boolean]; rightButtons:[boolean,boolean] }
const zero={x:0,y:0,force:0}
const snap=ref<Snapshot>({phase:'searching',message:'Поиск устройства…',transport:'usb',samples:0,left:zero,right:zero,inputMode:'touchpad',pressThreshold:0.30,hapticAmplitude:0.20,leftButtons:[false,false],rightButtons:[false,false]})
const mode=ref<Mode>('usb'),tab=ref<'mode'|'status'|'settings'>('status'),busy=ref(false)
const inputBusy=ref(false),inputError=ref('')
const linkError=ref('')
const tabs=[{id:'mode',label:'Режим'},{id:'status',label:'Статус'},{id:'settings',label:'Настройки'}] as const
let unlisten:UnlistenFn|undefined
const connected=computed(()=>snap.value.phase==='connected')
async function select(next:Mode){if(next===mode.value)return;mode.value=next;busy.value=true;try{await invoke('set_transport',{transport:next})}catch(e){snap.value={...snap.value,phase:'error',message:String(e)}}finally{busy.value=false}}
async function selectInputMode(next:InputMode){if(next===snap.value.inputMode)return;inputError.value='';if(!isTauri()){snap.value={...snap.value,inputMode:next,leftButtons:[false,false],rightButtons:[false,false]};return}inputBusy.value=true;try{await invoke('set_input_mode',{mode:next})}catch(e){inputError.value=String(e)}finally{inputBusy.value=false}}
async function savePressThreshold(threshold:number){inputError.value='';if(!isTauri()){snap.value={...snap.value,pressThreshold:threshold};return}inputBusy.value=true;try{await invoke('set_press_threshold',{threshold})}catch(e){inputError.value=String(e)}finally{inputBusy.value=false}}
async function saveHapticAmplitude(amplitude:number){inputError.value='';if(!isTauri()){snap.value={...snap.value,hapticAmplitude:amplitude};return}inputBusy.value=true;try{await invoke('set_haptic_amplitude',{amplitude})}catch(e){inputError.value=String(e)}finally{inputBusy.value=false}}
async function openModule(){linkError.value='';if(!isTauri()){window.open('https://github.com/Lateir/qptp-module','_blank','noopener,noreferrer');return}try{await invoke('open_module_page')}catch(e){linkError.value=String(e)}}
function status(side:'left'|'right'){const root=snap.value.status||{};const controllers=root.controllers as Record<string,unknown>|undefined;const value=root[side]||controllers?.[side];return (value&&typeof value==='object'?value:{}) as Record<string,unknown>}
function online(side:'left'|'right'){return connected.value&&status(side).connected===true}
function battery(side:'left'|'right'){const b=status(side).battery_percent??status(side).battery;return typeof b==='number'?`${Math.round(b)}%`:'—'}
onMounted(async()=>{if(!isTauri())return;unlisten=await listen<Snapshot>('stream-state',e=>snap.value=e.payload);snap.value=await invoke<Snapshot>('get_stream_state');mode.value=snap.value.transport})
onUnmounted(()=>unlisten?.())
</script>
<template>
<div class="app">
  <TouchPad v-bind="snap.left" :live="online('left')" :mode="snap.inputMode" :buttons="snap.leftButtons" :threshold="snap.pressThreshold"/>
  <main class="center">
    <h1>Quest Pro <em>touch</em> Plus</h1>
    <div class="view">
      <template v-if="tab==='status'">
        <p class="link" :title="snap.message"><span>{{mode==='usb'?'USB':'LAN'}}</span><b :class="snap.phase">{{snap.phase}}</b></p>
        <div class="cards">
          <div v-for="side in (['left','right'] as const)" :key="side" class="card">
            <h2>{{side==='left'?'Left':'Right'}}</h2>
            <p><i :class="{on:online(side)}"></i>{{online(side)?'connected':'offline'}}</p>
            <small>B: {{battery(side)}}</small>
          </div>
        </div>
      </template>
      <template v-else-if="tab==='mode'">
        <ModePanel :mode="snap.inputMode" :press-threshold="snap.pressThreshold" :haptic-amplitude="snap.hapticAmplitude" :busy="inputBusy" @select="selectInputMode" @threshold="savePressThreshold" @amplitude="saveHapticAmplitude"/>
        <p v-if="inputError" class="mode-error" role="alert">{{inputError}}</p>
      </template>
      <div v-else class="settings-view">
        <div class="modes">
          <button :class="{active:mode==='usb'}" :disabled="busy" @click="select('usb')"><b>USB</b><small>встроенный ADB</small></button>
          <button :class="{active:mode==='lan'}" :disabled="busy" @click="select('lan')"><b>LAN</b><small>автообнаружение</small></button>
        </div>
        <button type="button" class="module-link" @click="openModule">
          <b>Нужен запущенный qptp-module на Quest Pro</b>
          <span>github.com/Lateir/qptp-module ↗</span>
        </button>
        <p v-if="linkError" class="link-error" role="alert">{{linkError}}</p>
        <p class="meta credit">Lateir · v{{version}}</p>
      </div>
    </div>
    <nav class="tabs"><button v-for="t in tabs" :key="t.id" :class="{active:tab===t.id}" @click="tab=t.id">{{t.label}}</button></nav>
  </main>
  <TouchPad v-bind="snap.right" :live="online('right')" :mode="snap.inputMode" :buttons="snap.rightButtons" :threshold="snap.pressThreshold"/>
</div>
</template>
