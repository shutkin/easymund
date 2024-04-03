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
    <div class="cls_room_participant">
        <p>{{ participant.name }}</p>
        <span>🎤 {{ mic_state }}</span>
    </div>
</template>

<style>
    .cls_room_participant {padding: 0.5em; margin: 0.5em; height: fit-content; width: fit-content; border: solid 1px; border-color: lightgrey;}
</style>