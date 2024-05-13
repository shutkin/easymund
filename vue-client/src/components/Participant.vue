<script setup>
import { ref, watch } from 'vue';
import { room_state } from '../room_state';
import { event_bus } from '../event_bus';

const props = defineProps(['participant_id']);
const participant = ref(room_state.participants.find((p) => p.id == props.participant_id));
watch(
    () => room_state.participants,
    (participants) => {
        participant.value = participants.find((p) => p.id == props.participant_id);
    }
);
const is_participant_talking = ref(false);
watch(
    () => room_state.is_talking,
    (is_talking) => {
        console.log("talking clinets: " + is_talking + ", self id: " + participant.value.id);
        is_participant_talking.value = is_talking.includes(participant.value.id);
    }
);
</script>

<template>
    <div class="cls_room_participant" :style="is_participant_talking ? 'box-shadow: 0 0 0.25em rgba(0, 0, 0, 0.5);' : 'box-shadow: none;'">
        <div class="cls_participant_info">{{ participant.name }}</div>
        <div class="cls_participant_status">
            <div v-if="participant.is_admin" style="width: 1.5em; text-align: center; color: white;">A</div>
            <div v-if="!participant.is_muted" class="cls_icon_mic" style="width: 1.5em; margin: 0.25em;"></div>
            <div v-if="participant.is_muted" class="cls_icon_mic_muted" style="width: 1.5em; margin: 0.25em;"></div>
            <div v-if="participant.is_sharing" class="cls_icon_screen_share" style="width: 1.5em; margin: 0.25em;"></div>
        </div>
        <div v-if="room_state.is_admin && participant.id != room_state.self_id" class="cls_participant_ctrl">
            <button class="cls_button"
                @click="event_bus.fire({type: 'event_make_admin', data: props.participant_id})"
                style="width: fit-content; height: 2em; margin: 0.25em;">Дать админа</button>
            <button v-if="!participant.is_muted" class="cls_button"
                @click="event_bus.fire({type: 'event_make_muted', data: props.participant_id})"
                style="width: fit-content; height: 2em; margin: 0.25em;">Заглушить</button>
        </div>
    </div>
</template>

<style>
    .cls_room_participant {
        height: fit-content; width: fit-content;
        margin: 0.25em;
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
    .cls_participant_status {
        background-color: #add8e7;
        display: flex; flex-direction: row;
    }
    .cls_participant_ctrl {
        background-color: #f5f5dd;
    }
</style>