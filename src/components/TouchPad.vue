<script setup lang="ts">
import { computed } from 'vue'
const props=defineProps<{x:number;y:number;force:number;live:boolean}>()
const clamp=(v:number)=>Math.max(0,Math.min(1,v))
const nx=computed(()=>clamp(props.x/255)),ny=computed(()=>clamp(props.y/255)),nf=computed(()=>clamp(props.force))
// Dot grows from 12px (touch) to 48px (full press).
const size=computed(()=>12+nf.value*36)
</script>
<template>
<section class="pad" :class="{live}">
  <dl class="readout">
    <div><dt>X</dt><dd>{{nx.toFixed(2)}}</dd></div>
    <div><dt>Y</dt><dd>{{ny.toFixed(2)}}</dd></div>
    <div><dt>F</dt><dd>{{nf.toFixed(2)}}</dd></div>
  </dl>
  <div class="field"><span v-if="live" class="dot" :style="{left:nx*100+'%',top:ny*100+'%',width:size+'px',height:size+'px'}"></span></div>
</section>
</template>
