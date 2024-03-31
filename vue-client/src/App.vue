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
    audio.send_message({type: "mute", value: room_state.is_muted});
    console.log("Audio initialized");
    started.value = true;

    socket.send_message({type: "json", data: {event: "join", participant:{name: user_name, is_muted: room_state.is_muted}}});
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
    room_state.participants = [];
    room_state.is_muted = true;
    room_state.chat = [];
}

function ambience(ambience_select) {
    for (const ambience of room_state.ambiences) {
        if (ambience.name === ambience_select) {
            console.log(ambience);
            socket.send_message({type: "json", data: {event: "ambience", ambience: ambience.id}});
        }
    }
}

function mic_switch() {
    room_state.is_muted = !room_state.is_muted;
    audio.send_message({type: "mute", value: room_state.is_muted});
    socket.send_message({type: "json", data: {event: "participant", participant: {is_muted: room_state.is_muted}}});
}

function chat(text) {
    console.log("Chat " + text);
    socket.send_message({type: "json", data: {event: "chat", chat: {message: text}}});
}

function on_ws_message(event) {
    if (event.type === "stream" && audio != null) {
        audio.send_message(event);
    } else if (event.type === "log") {
        console.log("Socket log: " + event.message);
    } else if (event.type === "json") {
        const je = event.data;
        if (je.event === "room") {
            room_state.participants = je.participants;
            room_state.ambiences = je.ambiences;
            room_state.ambience = je.ambience;
            room_state.chat = je.chat.history;
        } else if (je.event === "participants") {
            room_state.participants = je.participants;
        } else if (je.event === "ambience") {
            room_state.ambience = je.ambience;
        } else if (je.event === "chat") {
            room_state.chat.push(je.chat.history.pop());
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

    <div class="cls_app">
        <Login v-if="!started" @event_login="start"/>
        <Room v-if="started" @event_leave="stop" @event_ambience="ambience" @event_mic="mic_switch" @event_chat="chat"/>
    </div>
</template>

<style>
</style>
