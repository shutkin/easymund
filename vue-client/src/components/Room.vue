<script setup>
import { ref, watch } from 'vue';
import Participant from './Participant.vue';
import { room_state } from '../room_state';

defineEmits(['event_leave', 'event_ambience']);
const ambience_select = ref("");
watch(() => room_state.ambience, (ambience_id) => {
    for (const ambience of room_state.ambiences) {
        if (ambience.id === ambience_id && ambience_select.value != ambience.name) {
            console.log("change ambience to " + ambience.name);
            ambience_select.value = ambience.name;
        }
    }
});
</script>
<template>
    <div class="wrapper">
        <label>Список участников:</label>
        <Participant v-for="(participant) in room_state.participants" :key="participant.name" :participant_name="participant.name"></Participant>
        <label>Фоновый звук:</label>
        <select v-model="ambience_select" @change="$emit('event_ambience', ambience_select)">
            <option v-for="(ambience) in room_state.ambiences" :key="ambience.id" :id="ambience.id">{{ambience.name}}</option>
        </select>
        <hr>
        <button @click="$emit('event_leave')">Выйти</button>
    </div>
</template>