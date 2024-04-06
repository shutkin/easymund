<script setup>
import { ref, watch } from 'vue';
import Chat from './Chat.vue';
import Participant from './Participant.vue';
import { room_state } from '../room_state';
import { event_bus } from '../event_bus';

const ambience_select = ref("");
watch(() => room_state.ambience, (ambience_id) => {
    for (const ambience of room_state.ambiences) {
        if (ambience.id === ambience_id && ambience_select.value != ambience.name) {
            console.log("change ambience to " + ambience.name);
            ambience_select.value = ambience.name;
        }
    }
});

function on_ambience() {
    event_bus.fire({type: "event_ambience", data: ambience_select.value});
}
</script>
<template>
    <section class="cls_main_cnt">
        <Chat/>
        <div class="cls_room_cnt">
            <div class="cls_room_participants">
                <Participant v-for="(participant) in room_state.participants" :key="participant.id" :participant="participant"/>
            </div>
            <div class="cls_room_controls">
                <button class="cls_button" style="width: 6em;" @click="event_bus.fire({type: 'event_mute', data: {}})">
                    <svg v-if="!room_state.is_muted" xmlns="http://www.w3.org/2000/svg" width="22" height="22" fill="currentColor" viewBox="0 0 16 16">
                        <path d="M3.5 6.5A.5.5 0 0 1 4 7v1a4 4 0 0 0 8 0V7a.5.5 0 0 1 1 0v1a5 5 0 0 1-4.5 4.975V15h3a.5.5 0 0 1 0 1h-7a.5.5 0 0 1 0-1h3v-2.025A5 5 0 0 1 3 8V7a.5.5 0 0 1 .5-.5"/>
                        <path d="M10 8a2 2 0 1 1-4 0V3a2 2 0 1 1 4 0zM8 0a3 3 0 0 0-3 3v5a3 3 0 0 0 6 0V3a3 3 0 0 0-3-3"/>
                    </svg>
                    <svg v-else xmlns="http://www.w3.org/2000/svg" width="22" height="22" fill="currentColor" viewBox="0 0 16 16">
                        <path d="M13 8c0 .564-.094 1.107-.266 1.613l-.814-.814A4 4 0 0 0 12 8V7a.5.5 0 0 1 1 0zm-5 4c.818 0 1.578-.245 2.212-.667l.718.719a5 5 0 0 1-2.43.923V15h3a.5.5 0 0 1 0 1h-7a.5.5 0 0 1 0-1h3v-2.025A5 5 0 0 1 3 8V7a.5.5 0 0 1 1 0v1a4 4 0 0 0 4 4m3-9v4.879l-1-1V3a2 2 0 0 0-3.997-.118l-.845-.845A3.001 3.001 0 0 1 11 3"/>
                        <path d="m9.486 10.607-.748-.748A2 2 0 0 1 6 8v-.878l-1-1V8a3 3 0 0 0 4.486 2.607m-7.84-9.253 12 12 .708-.708-12-12z"/>
                    </svg>
                </button>
                <div>
                    <span style="color: rgba(0, 0, 0, 0.75);">Фоновый звук:</span>
                    <select class="cls_select" v-model="ambience_select" @change="on_ambience">
                        <option v-for="(ambience) in room_state.ambiences" :key="ambience.id" :id="ambience.id">{{ambience.name}}</option>
                    </select>
                </div>
                <button class="cls_button" @click="event_bus.fire({type: 'event_leave', data: {}})">Выйти</button>
            </div>
        </div>
    </section>
</template>

<style>
    .cls_main_cnt {
        height: 97%; width: 90%;
        display: grid; grid-template-columns: 24em 1fr;
        background-color: #f1ead2;
        border-radius: 0.5em;
        box-shadow: 0 0.5em 1em rgba(0, 0, 0, 0.25);
    }
    .cls_room_cnt {display: grid; grid-template-rows: 1fr 3.5em;}
    .cls_room_participants {padding: 0.5em; display: flex; overflow: auto;}
    .cls_room_controls {
        padding: 0.5em;
        display: flex; flex-direction: row; gap: 0.5em;
        align-items: center;
        background-color: #f5f5dd;
    }
</style>