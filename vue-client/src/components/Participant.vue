<script setup>
import { ref, watch } from 'vue';

const props = defineProps(['participant'])
const mic_state = ref("выкл");
watch(
    () => props.participant,
    (p) => {mic_state.value = p.is_muted ? "выкл" : "вкл"},
    {deep: true}
);
</script>

<template>
    <div class="cls_div_participant">
        <p>{{ participant.name }}</p>
        <span>🎤 {{ mic_state }}</span>
    </div>
</template>

<style>
.cls_div_participant {
  border: solid 1px;
  margin: 1em;
}
</style>