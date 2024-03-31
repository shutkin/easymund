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
    <div class="div_participant">
        <p>{{ participant.name }}</p>
        <p>🎤 {{ mic_state }}</p>
    </div>
</template>

<style>
.div_participant {
  border: solid;
  margin: 1em;
}
</style>