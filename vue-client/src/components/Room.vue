<script setup>
import { ref, watch } from 'vue';
import Chat from './Chat.vue';
import Participant from './Participant.vue';
import { room_state } from '../room_state';
import { event_bus } from '../event_bus';

const ambience_select = ref("");
watch(() => room_state.ambience, (ambience_id) => {
    set_ambience(ambience_id);
});

function set_ambience(ambience_id) {
    for (const ambience of room_state.ambiences) {
        if (ambience.id === ambience_id && ambience_select.value != ambience.name) {
            console.log("change ambience to " + ambience.name);
            ambience_select.value = ambience.name;
        }
    }
}

function on_ambience() {
    event_bus.fire({type: "event_ambience", data: ambience_select.value});
}

set_ambience(room_state.ambience);
</script>
<template>
    <section class="cls_main_cnt">
        <Chat/>
        <div class="cls_room_cnt">
            <div class="cls_room_title">
                <h3>{{ room_state.name }}</h3>
            </div>
            <div class="cls_room_participants">
                <Participant v-for="(participant) in room_state.participants" :key="participant.id" :participant="participant"/>
            </div>
            <div class="cls_room_screen">
                <video id="video" autoplay="autoplay" style="width: 100%; height: 100%;"></video>
            </div>
            <div class="cls_room_controls">
                <button class="cls_button" style="width: 6em;" @click="event_bus.fire({type: 'event_mute', data: {}})">
                    <div v-if="!room_state.is_muted" class="cls_icon_mic"></div>
                    <div v-else class="cls_icon_mic_muted"></div>
                </button>
                <button v-if="room_state.is_admin" class="cls_button" style="width: 6em;" @click="event_bus.fire({type: 'event_screen', data: {}})">
                    <div v-if="!room_state.is_screen_sharing" class="cls_icon_screen_share"></div>
                    <div v-else class="cls_icon_screen_share_stop"></div>
                </button>
                <div v-if="room_state.is_admin">
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
    .cls_room_cnt {display: grid; grid-template-rows: 2.5em auto 1fr 3.5em;}
    .cls_room_title {text-align: center;}
    .cls_room_participants {padding: 0.5em; white-space: nowrap; overflow: auto;}
    .cls_room_screen {border: solid 1px #ddd; margin: 0.5em;}
    .cls_room_controls {
        padding: 0.5em;
        display: flex; flex-direction: row; gap: 0.5em;
        align-items: center;
        background-color: #f5f5dd;
    }
</style>