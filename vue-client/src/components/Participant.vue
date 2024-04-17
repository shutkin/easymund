<script setup>
import { ref, watch } from 'vue';

const props = defineProps(['participant'])
const mic_state = ref(props.participant.is_muted);
const screen_state = ref(props.participant.is_sharing);
const admin_state = ref(props.participant.is_admin);
watch(
    () => props.participant,
    (p) => {
        admin_state.value = p.is_admin;
        mic_state.value = !p.is_muted;
        screen_state.value = p.is_sharing;
    },
    {deep: true}
);
</script>

<template>
    <div class="cls_room_participant">
        <div class="cls_participant_info">{{ participant.name }}</div>
        <div class="cls_participant_ctrl">
            <div v-if="admin_state" style="width: 1.5em; text-align: center; color: white;">A</div>
            <div v-if="mic_state" class="cls_icon_mic" style="width: 1.5em; margin: 0.25em;"></div>
            <div v-if="!mic_state" class="cls_icon_mic_muted" style="width: 1.5em; margin: 0.25em;"></div>
            <div v-if="screen_state" class="cls_icon_screen_share" style="width: 1.5em; margin: 0.25em;"></div>
        </div>
    </div>
</template>

<style>
    .cls_room_participant {
        height: fit-content; width: fit-content;
        display: inline-block;
        border: 1px solid #ddd;
        border-radius: 5px;
        background-color: #f9f9f9;
    }
    .cls_room_participant:hover {
        background-color: #f0f0f0;
    }
    .cls_participant_info {
        padding: 0.5em;
        height: fit-content; width: fit-content;
    }
    .cls_participant_ctrl {
        background-color: #365194;
        display: flex; flex-direction: row;
    }
</style>