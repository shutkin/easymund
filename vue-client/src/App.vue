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
    room_state.event_bus.listen("ws_json", on_ws_json);
    room_state.event_bus.listen("ws_stream", on_ws_stream);
    room_state.event_bus.listen("audio_log", (event) => { console.log("Audio log: " + event.data); });
    room_state.event_bus.listen("audio_stream", on_audio_stream);
    room_state.event_bus.listen("event_chat", on_chat);
    room_state.event_bus.listen("event_ambience", on_ambience);
    room_state.event_bus.listen("event_mute", on_mic_switch);
    room_state.event_bus.listen("event_leave", on_leave);

    socket = new EasymundSocket("dev-room", room_state.event_bus);
    audio = new EasymundAudio(room_state.event_bus);
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

function on_ambience(event) {
    for (const ambience of room_state.ambiences) {
        if (ambience.name === event.data) {
            socket.send_message({type: "json", data: {event: "ambience", ambience: ambience.id}});
        }
    }
}

function on_mic_switch(event) {
    room_state.is_muted = !room_state.is_muted;
    audio.send_message({type: "audio_mute", value: room_state.is_muted});
    socket.send_message({type: "json", data: {event: "participant", participant: {is_muted: room_state.is_muted}}});
}

function on_chat(event) {
    socket.send_message({type: "json", data: {event: "chat", chat: {message: event.data}}});
}

function on_ws_stream(event) {
    if (audio != null) {
        audio.send_message({type: "audio_stream", data: event.data});
    }
}

function on_ws_json(event) {
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

function on_audio_stream(event) {
    if (socket != null) {
        socket.send_message(event);
    }
}
</script>

<template>
    <Login v-if="!started" @event_login="start"/>
    <Room v-if="started"/>
</template>

<style>
</style>
