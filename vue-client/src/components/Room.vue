<script setup>
import Participant from './Participant.vue';
import { room_state } from '../room_state';
defineEmits(['event_leave'])
</script>
<template>
    <div class="wrapper">
        <h3>Список участников:</h3>
        <Participant v-for="(participant) in room_state.participants" :key="participant.name" :participant_name="participant.name"></Participant>
        <button @click="$emit('event_leave')">Выйти</button>
    </div>
</template>