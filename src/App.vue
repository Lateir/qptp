<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { version } from '../package.json'
import ModePanel from '@/components/ModePanel.vue'
import TouchPad from '@/components/TouchPad.vue'
import { detectLanguage, isLanguage, languages, translate, type Language, type TranslationKey } from '@/i18n'
type Mode = 'usb' | 'lan'
type InputMode = 'touchpad' | 'button' | 'two_buttons'
type Side = { x:number; y:number; force:number }
type Snapshot = { phase:string; message:string; transport:Mode; endpoint?:string; protocol?:string; samples:number; left:Side; right:Side; status?:Record<string,unknown>; inputMode:InputMode; pressThreshold:number; hapticAmplitude:number; language:Language; leftButtons:[boolean,boolean]; rightButtons:[boolean,boolean]; steamvrConnected:boolean }
const zero={x:0,y:0,force:0}
const snap=ref<Snapshot>({phase:'searching',message:'Поиск устройства…',transport:'usb',samples:0,left:zero,right:zero,inputMode:'touchpad',pressThreshold:0.30,hapticAmplitude:0.20,language:'ru',leftButtons:[false,false],rightButtons:[false,false],steamvrConnected:false})
const mode=ref<Mode>('usb'),tab=ref<'mode'|'status'|'settings'>('status'),busy=ref(false)
const inputBusy=ref(false),inputError=ref('')
const linkError=ref(''),settingsError=ref(''),languageBusy=ref(false)
const storedLanguage=localStorage.getItem('qptp-language')||''
const language=ref<Language>(isLanguage(storedLanguage)?storedLanguage:detectLanguage(navigator.languages?.length?navigator.languages:[navigator.language]))
const t=(key:TranslationKey)=>translate(language.value,key)
const tabs=[{id:'mode',label:'tabMode'},{id:'status',label:'tabStatus'},{id:'settings',label:'tabSettings'}] as const
let unlisten:UnlistenFn|undefined
const connected=computed(()=>snap.value.phase==='connected')
const phaseLabel=computed(()=>t((['searching','connecting','connected','error'].includes(snap.value.phase)?snap.value.phase:'error') as TranslationKey))
const phaseHint=computed(()=>t((snap.value.phase==='connected'?'connectedHint':snap.value.phase==='connecting'?'connectingHint':snap.value.phase==='searching'?'searchingHint':'errorHint')))
watch(language,value=>{document.documentElement.lang=value;document.documentElement.dir=value==='ar'?'rtl':'ltr';localStorage.setItem('qptp-language',value)},{immediate:true})
async function select(next:Mode){if(next===mode.value)return;mode.value=next;busy.value=true;try{await invoke('set_transport',{transport:next})}catch(e){snap.value={...snap.value,phase:'error',message:String(e)}}finally{busy.value=false}}
async function selectInputMode(next:InputMode){if(next===snap.value.inputMode)return;inputError.value='';if(!isTauri()){snap.value={...snap.value,inputMode:next,leftButtons:[false,false],rightButtons:[false,false]};return}inputBusy.value=true;try{await invoke('set_input_mode',{mode:next})}catch(e){inputError.value=String(e)}finally{inputBusy.value=false}}
async function savePressThreshold(threshold:number){inputError.value='';if(!isTauri()){snap.value={...snap.value,pressThreshold:threshold};return}inputBusy.value=true;try{await invoke('set_press_threshold',{threshold})}catch(e){inputError.value=String(e)}finally{inputBusy.value=false}}
async function saveHapticAmplitude(amplitude:number){inputError.value='';if(!isTauri()){snap.value={...snap.value,hapticAmplitude:amplitude};return}inputBusy.value=true;try{await invoke('set_haptic_amplitude',{amplitude})}catch(e){inputError.value=String(e)}finally{inputBusy.value=false}}
async function changeLanguage(event:Event){const next=(event.target as HTMLSelectElement).value;if(!isLanguage(next)||next===language.value)return;settingsError.value='';languageBusy.value=true;try{if(isTauri())await invoke('set_language',{language:next});language.value=next;snap.value={...snap.value,language:next}}catch{settingsError.value=t('saveError');(event.target as HTMLSelectElement).value=language.value}finally{languageBusy.value=false}}
async function openModule(){linkError.value='';if(!isTauri()){window.open('https://github.com/Lateir/qptp-module','_blank','noopener,noreferrer');return}try{await invoke('open_module_page')}catch(e){linkError.value=String(e)}}
function status(side:'left'|'right'){const root=snap.value.status||{};const controllers=root.controllers as Record<string,unknown>|undefined;const value=root[side]||controllers?.[side];return (value&&typeof value==='object'?value:{}) as Record<string,unknown>}
function online(side:'left'|'right'){return connected.value&&status(side).connected===true}
function battery(side:'left'|'right'){const b=status(side).battery_percent??status(side).battery;return typeof b==='number'?`${Math.round(b)}%`:'—'}
onMounted(async()=>{if(!isTauri())return;unlisten=await listen<Snapshot>('stream-state',e=>{snap.value=e.payload;if(isLanguage(e.payload.language))language.value=e.payload.language});snap.value=await invoke<Snapshot>('get_stream_state');mode.value=snap.value.transport;if(isLanguage(snap.value.language))language.value=snap.value.language})
onUnmounted(()=>unlisten?.())
</script>
<template>
<div class="app">
  <TouchPad v-bind="snap.left" :live="online('left')" :mode="snap.inputMode" :buttons="snap.leftButtons" :threshold="snap.pressThreshold" :language="language"/>
  <main class="center">
    <h1>Quest Pro <em>touch</em> Plus</h1>
    <div class="view">
      <template v-if="tab==='status'">
        <p class="link" :title="phaseHint"><span>{{mode==='usb'?'USB':'LAN'}}</span><b :class="snap.phase">{{phaseLabel}}</b></p>
        <p class="link steamvr-link"><span>SteamVR</span><b :class="{connected:snap.steamvrConnected}">{{snap.steamvrConnected?t('online'):t('offline')}}</b></p>
        <div class="cards">
          <div v-for="side in (['left','right'] as const)" :key="side" class="card">
            <h2>{{t(side)}}</h2>
            <p><i :class="{on:online(side)}"></i>{{online(side)?t('online'):t('offline')}}</p>
            <small>{{t('battery')}}: {{battery(side)}}</small>
          </div>
        </div>
      </template>
      <template v-else-if="tab==='mode'">
        <ModePanel :mode="snap.inputMode" :press-threshold="snap.pressThreshold" :haptic-amplitude="snap.hapticAmplitude" :busy="inputBusy" :language="language" @select="selectInputMode" @threshold="savePressThreshold" @amplitude="saveHapticAmplitude"/>
        <p v-if="inputError" class="mode-error" role="alert">{{t('saveError')}}</p>
      </template>
      <div v-else class="settings-view">
        <div class="modes">
          <button :class="{active:mode==='usb'}" :disabled="busy" @click="select('usb')"><b>USB</b><small>{{t('bundledAdb')}}</small></button>
          <button :class="{active:mode==='lan'}" :disabled="busy" @click="select('lan')"><b>LAN</b><small>{{t('autoDiscovery')}}</small></button>
        </div>
        <div class="language-control"><label for="app-language">{{t('language')}}</label><select id="app-language" :value="language" :disabled="languageBusy" @change="changeLanguage"><option v-for="item in languages" :key="item.code" :value="item.code">{{item.name}}</option></select></div>
        <button type="button" class="module-link" @click="openModule">
          <b>{{t('moduleRequired')}}</b>
          <span>github.com/Lateir/qptp-module ↗</span>
        </button>
        <p v-if="linkError||settingsError" class="link-error" role="alert">{{linkError?t('openError'):settingsError}}</p>
        <p class="meta credit">Lateir · v{{version}}</p>
      </div>
    </div>
    <nav class="tabs"><button v-for="item in tabs" :key="item.id" :class="{active:tab===item.id}" @click="tab=item.id">{{t(item.label)}}</button></nav>
  </main>
  <TouchPad v-bind="snap.right" :live="online('right')" :mode="snap.inputMode" :buttons="snap.rightButtons" :threshold="snap.pressThreshold" :language="language"/>
</div>
</template>
