<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { version } from '../package.json'
import ModePanel from '@/components/ModePanel.vue'
import SensorsPanel from '@/components/SensorsPanel.vue'
import TouchPad from '@/components/TouchPad.vue'
import { detectLanguage, isLanguage, languages, translate, type Language, type TranslationKey } from '@/i18n'
type Mode = 'usb' | 'lan'
type InputMode = 'touchpad' | 'button' | 'two_buttons'
type Side = { x:number; y:number; force:number; stylus:number; triggerProximity:number; triggerSlide:number }
type Snapshot = { phase:string; message:string; transport:Mode; endpoint?:string; protocol?:string; moduleVersion?:{version:string;versionCode:number}; minimumModuleVersion:string; moduleUpdateRequired:boolean; samples:number; left:Side; right:Side; status?:Record<string,unknown>; inputMode:InputMode; pressThreshold:number; releaseThreshold:number; hapticAmplitude:number; language:Language; leftButtons:[boolean,boolean]; rightButtons:[boolean,boolean]; steamvrConnected:boolean; steamvrLifecycle:boolean }
const zero={x:0,y:0,force:0,stylus:0,triggerProximity:0,triggerSlide:0}
const snap=ref<Snapshot>({phase:'searching',message:'Поиск устройства…',transport:'lan',minimumModuleVersion:'v3.3',moduleUpdateRequired:false,samples:0,left:zero,right:zero,inputMode:'touchpad',pressThreshold:0.30,releaseThreshold:0.20,hapticAmplitude:0.20,language:'ru',leftButtons:[false,false],rightButtons:[false,false],steamvrConnected:false,steamvrLifecycle:true})
const mode=ref<Mode>('lan'),tab=ref<'sensors'|'mode'|'status'|'settings'|'updates'>('status'),busy=ref(false)
const inputBusy=ref(false),inputError=ref('')
const linkError=ref(''),settingsError=ref(''),languageBusy=ref(false),lifecycleBusy=ref(false)
const storedLanguage=localStorage.getItem('qptp-language')||''
const language=ref<Language>(isLanguage(storedLanguage)?storedLanguage:detectLanguage(navigator.languages?.length?navigator.languages:[navigator.language]))
const t=(key:TranslationKey)=>translate(language.value,key)
const tabs=[{id:'sensors',label:'tabSensors'},{id:'mode',label:'tabMode'},{id:'status',label:'tabStatus'},{id:'settings',label:'tabSettings'}] as const
let unlisten:UnlistenFn|undefined
let updateTimer:ReturnType<typeof setInterval>|undefined
const latestApp=ref<string>(),latestModule=ref<string>()
function versionParts(value:string){const match=/^v?(\d+)\.(\d+)(?:\.(\d+))?$/.exec(value);return match?[Number(match[1]),Number(match[2]),Number(match[3]||0)]:null}
function newer(candidate:string|undefined,current:string|undefined){if(!candidate||!current)return false;const a=versionParts(candidate),b=versionParts(current);if(!a||!b)return false;return a.some((part,index)=>part!==b[index]&&a.slice(0,index).every((prior,i)=>prior===b[i])&&part>b[index])}
const appUpdate=computed(()=>newer(latestApp.value,version))
const moduleUpdate=computed(()=>!snap.value.moduleUpdateRequired&&newer(latestModule.value,snap.value.moduleVersion?.version))
const availableUpdates=computed(()=>appUpdate.value||moduleUpdate.value)
const compatibleModuleRelease=computed(()=>!!latestModule.value&&(latestModule.value===snap.value.minimumModuleVersion||newer(latestModule.value,snap.value.minimumModuleVersion)))
watch(availableUpdates,available=>{if(!available&&tab.value==='updates')tab.value='status'})
async function latestRelease(repo:'qptp'|'qptp-module'){
  try{const response=await fetch(`https://api.github.com/repos/Lateir/${repo}/releases/latest`,{headers:{Accept:'application/vnd.github+json'},cache:'no-store',signal:AbortSignal.timeout(6000)});if(!response.ok)return;const data:unknown=await response.json();if(data&&typeof data==='object'&&'tag_name' in data&&typeof data.tag_name==='string'&&versionParts(data.tag_name))return data.tag_name}catch{/* Offline or rate limited: keep the last known result. */}
}
async function checkReleases(){const [app,module]=await Promise.all([latestRelease('qptp'),latestRelease('qptp-module')]);if(app)latestApp.value=app;if(module)latestModule.value=module}
const connected=computed(()=>snap.value.phase==='connected')
const phaseLabel=computed(()=>t((['searching','connecting','connected','error'].includes(snap.value.phase)?snap.value.phase:'error') as TranslationKey))
const phaseHint=computed(()=>t((snap.value.phase==='connected'?'connectedHint':snap.value.phase==='connecting'?'connectingHint':snap.value.phase==='searching'?'searchingHint':'errorHint')))
watch(language,value=>{document.documentElement.lang=value;document.documentElement.dir=value==='ar'?'rtl':'ltr';localStorage.setItem('qptp-language',value)},{immediate:true})
async function select(next:Mode){if(next===mode.value)return;mode.value=next;busy.value=true;try{await invoke('set_transport',{transport:next})}catch(e){snap.value={...snap.value,phase:'error',message:String(e)}}finally{busy.value=false}}
async function selectInputMode(next:InputMode){if(next===snap.value.inputMode)return;inputError.value='';if(!isTauri()){snap.value={...snap.value,inputMode:next,leftButtons:[false,false],rightButtons:[false,false]};return}inputBusy.value=true;try{await invoke('set_input_mode',{mode:next})}catch(e){inputError.value=String(e)}finally{inputBusy.value=false}}
async function saveButtonThresholds(press:number,release:number){inputError.value='';if(!isTauri()){snap.value={...snap.value,pressThreshold:press,releaseThreshold:release};return}inputBusy.value=true;try{await invoke('set_button_thresholds',{press,release})}catch(e){inputError.value=String(e)}finally{inputBusy.value=false}}
async function saveHapticAmplitude(amplitude:number){inputError.value='';if(!isTauri()){snap.value={...snap.value,hapticAmplitude:amplitude};return}inputBusy.value=true;try{await invoke('set_haptic_amplitude',{amplitude})}catch(e){inputError.value=String(e)}finally{inputBusy.value=false}}
async function changeLanguage(event:Event){const next=(event.target as HTMLSelectElement).value;if(!isLanguage(next)||next===language.value)return;settingsError.value='';languageBusy.value=true;try{if(isTauri())await invoke('set_language',{language:next});language.value=next;snap.value={...snap.value,language:next}}catch{settingsError.value=t('saveError');(event.target as HTMLSelectElement).value=language.value}finally{languageBusy.value=false}}
async function changeSteamvrLifecycle(event:Event){const enabled=(event.target as HTMLInputElement).checked;settingsError.value='';lifecycleBusy.value=true;try{if(isTauri())await invoke('set_steamvr_lifecycle',{enabled});snap.value={...snap.value,steamvrLifecycle:enabled}}catch{settingsError.value=t('saveError');(event.target as HTMLInputElement).checked=snap.value.steamvrLifecycle}finally{lifecycleBusy.value=false}}
async function openModule(){linkError.value='';if(!isTauri()){window.open('https://github.com/Lateir/qptp-module','_blank','noopener,noreferrer');return}try{await invoke('open_module_page')}catch(e){linkError.value=String(e)}}
async function openRelease(target:'module'|'app'){linkError.value='';if(!isTauri()){window.open(`https://github.com/Lateir/${target==='app'?'qptp':'qptp-module'}/releases/latest`,'_blank','noopener,noreferrer');return}try{await invoke('open_release_page',{target})}catch(e){linkError.value=String(e)}}
function status(side:'left'|'right'){const root=snap.value.status||{};const controllers=root.controllers as Record<string,unknown>|undefined;const value=root[side]||controllers?.[side];return (value&&typeof value==='object'?value:{}) as Record<string,unknown>}
function online(side:'left'|'right'){return connected.value&&status(side).connected===true}
function battery(side:'left'|'right'){const b=status(side).battery_percent??status(side).battery;return typeof b==='number'?`${Math.round(b)}%`:'—'}
onMounted(async()=>{void checkReleases();updateTimer=setInterval(()=>void checkReleases(),60*60*1000);if(!isTauri())return;unlisten=await listen<Snapshot>('stream-state',e=>{snap.value=e.payload;if(isLanguage(e.payload.language))language.value=e.payload.language});snap.value=await invoke<Snapshot>('get_stream_state');mode.value=snap.value.transport;if(isLanguage(snap.value.language))language.value=snap.value.language})
onUnmounted(()=>{unlisten?.();if(updateTimer)clearInterval(updateTimer)})
</script>
<template>
<div class="app" :class="{'sensors-active':tab==='sensors'}">
  <TouchPad v-if="tab!=='sensors'" v-bind="snap.left" :live="online('left')" :mode="snap.inputMode" :buttons="snap.leftButtons" :threshold="snap.pressThreshold" :release-threshold="snap.releaseThreshold" :language="language"/>
  <main class="center">
    <h1 v-if="tab!=='settings'">Quest Pro <em>touch</em> Plus</h1>
    <div class="view">
      <template v-if="tab==='status'">
        <p class="link" :title="phaseHint"><span>{{mode==='usb'?'USB':'LAN'}}</span><b :class="snap.phase">{{phaseLabel}}</b></p>
        <p class="link steamvr-link"><span>SteamVR</span><b :class="{connected:snap.steamvrConnected}">{{snap.steamvrConnected?t('online'):t('offline')}}</b></p>
        <div v-if="snap.moduleUpdateRequired" class="update-card" role="alert">
          <b>{{t('updateRequired')}}</b>
          <small>{{t('magiskUpdateHint')}}</small>
          <small v-if="!compatibleModuleRelease">{{t('compatibleReleasePending')}} ({{snap.minimumModuleVersion}})</small>
          <button v-if="compatibleModuleRelease" type="button" @click="openRelease('module')">github.com/Lateir/qptp-module ↗</button>
        </div>
        <div v-else class="cards">
          <div v-for="side in (['left','right'] as const)" :key="side" class="card">
            <h2>{{t(side)}}</h2>
            <p><i :class="{on:online(side)}"></i>{{online(side)?t('online'):t('offline')}}</p>
            <small>{{t('battery')}}: {{battery(side)}}</small>
          </div>
        </div>
      </template>
      <SensorsPanel v-else-if="tab==='sensors'" :left="snap.left" :right="snap.right" :left-online="online('left')" :right-online="online('right')" :language="language"/>
      <template v-else-if="tab==='mode'">
        <ModePanel :mode="snap.inputMode" :press-threshold="snap.pressThreshold" :release-threshold="snap.releaseThreshold" :haptic-amplitude="snap.hapticAmplitude" :busy="inputBusy" :language="language" @select="selectInputMode" @thresholds="saveButtonThresholds" @amplitude="saveHapticAmplitude"/>
        <p v-if="inputError" class="mode-error" role="alert">{{t('saveError')}}</p>
      </template>
      <div v-else-if="tab==='updates'" class="updates-view">
        <h2>{{t('updateAvailable')}}</h2>
        <button v-if="moduleUpdate" type="button" class="release-row" @click="openRelease('module')">
          <b>{{t('moduleLabel')}}</b><span>{{snap.moduleVersion?.version}} → {{latestModule}} ↗</span>
        </button>
        <button v-if="appUpdate" type="button" class="release-row" @click="openRelease('app')">
          <b>{{t('appLabel')}}</b><span>v{{version}} → {{latestApp}} ↗</span>
        </button>
        <p v-if="linkError" class="link-error" role="alert">{{t('openError')}}</p>
      </div>
      <div v-else class="settings-view">
        <div class="modes">
          <button :class="{active:mode==='usb'}" :disabled="busy" @click="select('usb')"><b>USB</b><small>{{t('bundledAdb')}}</small></button>
          <button :class="{active:mode==='lan'}" :disabled="busy" @click="select('lan')"><b>LAN</b><small>{{t('autoDiscovery')}}</small></button>
        </div>
        <div class="language-control"><label for="app-language">{{t('language')}}</label><select id="app-language" :value="language" :disabled="languageBusy" @change="changeLanguage"><option v-for="item in languages" :key="item.code" :value="item.code">{{item.name}}</option></select></div>
        <label class="steamvr-lifecycle"><span>{{t('steamvrLifecycle')}}</span><input type="checkbox" role="switch" :checked="snap.steamvrLifecycle" :disabled="lifecycleBusy" @change="changeSteamvrLifecycle"/></label>
        <button type="button" class="module-link" @click="openModule">
          <b>{{t('moduleRequired')}}</b>
          <span>github.com/Lateir/qptp-module ↗</span>
        </button>
        <p v-if="linkError||settingsError" class="link-error" role="alert">{{linkError?t('openError'):settingsError}}</p>
        <p class="meta credit version-line">QPTP v{{version}}<template v-if="snap.moduleVersion"> · {{t('moduleLabel')}} {{snap.moduleVersion.version}}</template></p>
        <p class="meta credit">by lateir with ❤️</p>
      </div>
    </div>
    <nav class="navigation">
      <div class="tabs"><button v-for="item in tabs" :key="item.id" :class="{active:tab===item.id}" @click="tab=item.id">{{t(item.label)}}</button></div>
      <button v-if="availableUpdates" type="button" class="update-shortcut" :class="{active:tab==='updates'}" :aria-label="t('updateAvailable')" :title="t('updateAvailable')" @click="tab='updates'">↑</button>
    </nav>
  </main>
  <TouchPad v-if="tab!=='sensors'" v-bind="snap.right" :live="online('right')" :mode="snap.inputMode" :buttons="snap.rightButtons" :threshold="snap.pressThreshold" :release-threshold="snap.releaseThreshold" :language="language"/>
</div>
</template>
