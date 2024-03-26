<script setup>
import Room from './components/Room.vue';
import Login from './components/Login.vue';
import { EasymundAudio } from './audio';
import { EasymundSocket } from './ws';
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

function on_ws_message(event) {
    if (event.type === "stream" && audio != null) {
        audio.send_message(event);
    } else if (event.type === "log") {
        console.log("Socket log: " + event.message);
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
        <Room v-if="started" @event_leave="stop"/>
    </main>
</template>

<style scoped>
</style>
