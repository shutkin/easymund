<script setup>
import { ref, watch } from 'vue';

const props = defineProps(['participant'])
const mic_state = ref(false);
watch(
    () => props.participant,
    (p) => {mic_state.value = !p.is_muted},
    {deep: true}
);
</script>

<template>
    <div class="cls_room_participant">
        <p>{{ participant.name }}</p>
        <span>
            <div v-if="mic_state" class="cls_icon_mic"></div>
            <div v-else class="cls_icon_mic_muted"></div>
        </span>
    </div>
</template>

<style>
    .cls_room_participant {
        height: fit-content; width: fit-content;
        display: inline-block;
        padding: 1em; margin: 0.5em;
        border: 1px solid #ddd;
        border-radius: 5px;
        background-color: #f9f9f9;
    }
    .cls_room_participant:hover {
        background-color: #f0f0f0;
    }
</style>