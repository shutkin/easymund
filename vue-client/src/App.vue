<script setup>
import Room from './components/Room.vue';
import Login from './components/Login.vue';
import { EasymundAudio } from './audio';
import { EasymundSocket } from './ws';
import { room_state } from './room_state';
import { ref } from 'vue';

const started = ref(false);
/**
 * @type EasymundSocket
 */
var socket = null;
/**
 * @type EasymundAudio
 */
var audio = null;

async function start(user_name) {
    socket = new EasymundSocket("dev-room", on_ws_message);
    audio = new EasymundAudio(on_audio_message);
    await audio.init();
    console.log("Audio initialized");
    started.value = true;

    socket.send_message({type: "json", data: {event: "join", name: user_name}});
}

function stop() {
    if (started.value) {
        if (socket != null) {
            socket.close();
            socket = null;
        }
        if (audio != null) {
            audio.close();
            audio = null;
        }
        started.value = false;
    }
}

function ambience(ambience_select) {
    for (const ambience of room_state.ambiences) {
        if (ambience.name === ambience_select) {
            console.log(ambience);
            socket.send_message({type: "json", data: {event: "ambience", ambience: ambience.id}});
        }
    }
}

function on_ws_message(event) {
    if (event.type === "stream" && audio != null) {
        audio.send_message(event);
    } else if (event.type === "log") {
        console.log("Socket log: " + event.message);
    } else if (event.type === "json") {
        if (event.data.event === "room") {
            room_state.participants = event.data.participants;
            room_state.ambiences = event.data.ambiences;
        }
    }
}

function on_audio_message(event) {
    if (event.type === "stream" && socket != null) {
        socket.send_message(event);
    } else if (event.type === "log") {
        console.log("Audio log: " + event.message);
    }
}
</script>

<template>
    <header>
        <h1>Easymund</h1>
    </header>

    <main>
        <Login v-if="!started" @event_login="start"/>
        <Room v-if="started" @event_leave="stop" @event_ambience="ambience"/>
    </main>
</template>

<style scoped>
</style>
