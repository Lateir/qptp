<script setup lang="ts">
import { computed } from 'vue'
import { translate, type Language, type TranslationKey } from '@/i18n'
const props=defineProps<{x:number;y:number;force:number;live:boolean;mode:'touchpad'|'button'|'two_buttons';buttons:[boolean,boolean];threshold:number;releaseThreshold:number;language:Language}>()
const t=(key:TranslationKey)=>translate(props.language,key)
const clamp=(v:number)=>Math.max(0,Math.min(1,v))
const nx=computed(()=>clamp(props.x/255)),ny=computed(()=>clamp(props.y/255)),nf=computed(()=>clamp(props.force))
// Rust flips the raw Y axis, so the module's idle (0, 0) reaches the UI as (0, 255).
const touching=computed(()=>props.live&&(props.x!==0||props.y!==255))
// Dot grows from 12px (touch) to 48px (full press).
const size=computed(()=>12+nf.value*36)
const active=computed(()=>props.live&&props.buttons.some(Boolean))
</script>
<template>
<section class="pad" :class="{live,'button-pad':mode!=='touchpad'}">
  <dl v-if="mode==='touchpad'" class="readout">
    <div><dt>X</dt><dd>{{nx.toFixed(2)}}</dd></div>
    <div><dt>Y</dt><dd>{{ny.toFixed(2)}}</dd></div>
    <div><dt>F</dt><dd>{{nf.toFixed(2)}}</dd></div>
  </dl>
  <template v-else>
    <div class="pad-force"><span>{{t('force')}}</span><b>{{nf.toFixed(2)}}</b></div>
    <div v-if="mode==='button'" class="pad-button" :class="{pressed:active}"><b>1</b></div>
    <div v-else class="pad-zones"><div :class="{pressed:live&&buttons[0]}"><b>1</b><span>{{t('upper')}}</span></div><div :class="{pressed:live&&buttons[1]}"><b>2</b><span>{{t('lower')}}</span></div></div>
    <div class="pad-threshold">{{releaseThreshold.toFixed(2)}} / {{threshold.toFixed(2)}}</div>
  </template>
  <div v-if="mode==='touchpad'" class="field"><span v-if="touching" class="dot" :style="{left:nx*100+'%',top:ny*100+'%',width:size+'px',height:size+'px'}"></span></div>
</section>
</template>
