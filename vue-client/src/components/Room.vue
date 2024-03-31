<script setup>
import { ref, watch } from 'vue';
import Participant from './Participant.vue';
import { room_state } from '../room_state';

defineEmits(['event_leave', 'event_ambience', 'event_mic']);
const ambience_select = ref("");
watch(() => room_state.ambience, (ambience_id) => {
    for (const ambience of room_state.ambiences) {
        if (ambience.id === ambience_id && ambience_select.value != ambience.name) {
            console.log("change ambience to " + ambience.name);
            ambience_select.value = ambience.name;
        }
    }
});
const mic_state = ref(room_state.is_muted ? "выкл" : "вкл");
watch(() => room_state.is_muted, (is_muted) => {
    mic_state.value = is_muted ? "выкл" : "вкл";
});
</script>
<template>
    <div class="wrapper">
        <label for="div_participants">Участники:</label>
        <div id="div_participants" class="div_participants">
            <Participant v-for="(participant) in room_state.participants" :key="participant.name" :participant="participant"></Participant>
        </div>
        <label>Фоновый звук:</label>
        <select v-model="ambience_select" @change="$emit('event_ambience', ambience_select)">
            <option v-for="(ambience) in room_state.ambiences" :key="ambience.id" :id="ambience.id">{{ambience.name}}</option>
        </select>
        <hr>
        <label for="button_mic">🎤</label>
        <button id="button_mic" @click="$emit('event_mic')">{{mic_state}}</button>
        <button @click="$emit('event_leave')">Выйти</button>
    </div>
</template>

<style>
.div_participants {
    display: flex;
}
</style>