<script setup>
import Room from './components/Room.vue';
import Login from './components/Login.vue';
import { EasymundAudio } from './audio';
import { EasymundSocket } from './ws';
import { room_state } from './room_state';
import { event_bus } from './event_bus';
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
    event_bus.listen("ws_json", on_ws_json);
    event_bus.listen("ws_stream", on_ws_stream);
    event_bus.listen("audio_log", (event) => { console.log("Audio log: " + event.data); });
    event_bus.listen("audio_stream", on_audio_stream);
    event_bus.listen("event_chat", on_chat);
    event_bus.listen("event_ambience", on_ambience);
    event_bus.listen("event_mute", on_mic_switch);
    event_bus.listen("event_leave", on_leave);

    socket = new EasymundSocket("dev-room");
    audio = new EasymundAudio();
    await audio.init();
    audio.send_message({type: "audio_mute", value: room_state.is_muted});
    started.value = true;

    socket.send_message({type: "json", data: {event: "join", participant:{name: user_name, is_muted: room_state.is_muted}}});
}

function on_leave() {
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
    room_state.ambience = "";
}

function on_ambience(data) {
    for (const ambience of room_state.ambiences) {
        if (ambience.name === data) {
            socket.send_message({type: "json", data: {event: "ambience", ambience: ambience.id}});
        }
    }
}

function on_mic_switch() {
    room_state.is_muted = !room_state.is_muted;
    audio.send_message({type: "audio_mute", value: room_state.is_muted});
    socket.send_message({type: "json", data: {event: "participant", participant: {is_muted: room_state.is_muted}}});
}

function on_chat(data) {
    socket.send_message({type: "json", data: {event: "chat", chat: {message: data}}});
}

function on_ws_stream(data) {
    if (audio != null) {
        audio.send_message({type: "audio_stream", data: data});
    }
}

function on_ws_json(data) {
    if (data.event === "room") {
        room_state.participants = data.participants;
        room_state.ambiences = data.ambiences;
        room_state.ambience = data.ambience;
        room_state.chat = data.chat.history;
    } else if (data.event === "participants") {
        room_state.participants = data.participants;
    } else if (data.event === "ambience") {
        room_state.ambience = data.ambience;
    } else if (data.event === "chat") {
        room_state.chat.push(data.chat.history.pop());
    }
}

function on_audio_stream(data) {
    if (socket != null) {
        socket.send_message({type: "stream", data});
    }
}
</script>

<template>
    <Login v-if="!started" @event_login="start"/>
    <Room v-if="started"/>
</template>

<style>
</style>
