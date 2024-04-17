<script setup>
import Room from './components/Room.vue';
import Login from './components/Login.vue';
import Create from './components/Create.vue';
import { EasymundAudio } from './audio';
import { EasymundScreenShare } from './screen_share';
import { EasymundSocket } from './ws';
import { room_state } from './room_state';
import { event_bus } from './event_bus';
import { ref } from 'vue';

const room_id = ref(window.location.hash.split('#').pop());
const started = ref(false);
/**
 * @type EasymundSocket
 */
var socket = null;
/**
 * @type EasymundAudio
 */
var audio = null;
/**
 * @type EasymundScreenShare
 */
var screen_share = new EasymundScreenShare();
/**
 * @type boolean
 */
var is_error = false;

async function create(room_name) {
    const result = await postRequest("/create", {"name": room_name});
    const resp = JSON.parse(result);
    console.log(resp);
    room_id.value = resp.room_id;
    window.location.hash = resp.room_id;
}

/**
 * @param {String} url 
 * @param {*} data 
 */
function postRequest(url, data) {
    return new Promise(function (resolve, reject) {
        let xhr = new XMLHttpRequest();
        xhr.open("POST", url);
        xhr.onload = function () {
            if (this.status >= 200 && this.status < 300) {
                resolve(xhr.response);
            } else {
                reject({
                    status: this.status,
                    statusText: xhr.statusText
                });
            }
        };
        xhr.onerror = function () {
            reject({
                status: this.status,
                statusText: xhr.statusText
            });
        };
        xhr.send(JSON.stringify(data));
    });
}

async function start(user_name) {
    event_bus.listen("ws_json", on_ws_json);
    event_bus.listen("ws_audio", on_ws_audio);
    event_bus.listen("ws_video", on_ws_video);
    event_bus.listen("audio_log", (data) => { console.log("Audio log: " + data); });
    event_bus.listen("audio_stream", on_audio_stream);
    event_bus.listen("video_stream", on_video_stream);
    event_bus.listen("event_chat", on_chat);
    event_bus.listen("event_screen", on_screen);
    event_bus.listen("event_ambience", on_ambience);
    event_bus.listen("event_mute", on_mic_switch);
    event_bus.listen("event_leave", on_leave);

    audio = new EasymundAudio();
    await audio.init();
    audio.send_message({type: "audio_mute", value: room_state.is_muted});

    socket = new EasymundSocket(room_id.value);
    socket.send_message({type: "json", data: {event: "join", participant:{name: user_name, is_muted: room_state.is_muted}}});
}

function on_leave() {
    if (started.value) {
        started.value = false;
    }
    if (socket != null) {
        socket.close();
        socket = null;
    }
    if (audio != null) {
        audio.close();
        audio = null;
    }
    room_state.participants = [];
    room_state.is_muted = true;
    room_state.chat = [];
    room_state.ambience = "";
}

function send_participant() {
    const participant = {
        is_muted: room_state.is_muted,
        is_sharing: room_state.is_screen_sharing
    };
    socket.send_message({type: "json", data: {event: "participant", participant}});
}

async function on_screen() {
    if (!room_state.is_screen_sharing) {
        await screen_share.start_share();
        room_state.is_screen_sharing = true;
    } else {
        screen_share.stop_share();
        room_state.is_screen_sharing = false;
    }
    send_participant();
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
    send_participant();
}

function on_chat(data) {
    socket.send_message({type: "json", data: {event: "chat", chat: {message: data}}});
}

function on_ws_audio(data) {
    if (audio != null) {
        audio.send_message({type: "audio_stream", data: data});
    }
}

function on_ws_video(data) {
    screen_share.video_frame(data);
}

function on_ws_json(data) {
    if (data.event === "room") {
        if (data.participant) {
            console.log("myself: " + JSON.stringify(data.participant));
            room_state.self_id = data.participant.id;
            room_state.is_admin = data.participant.is_admin;
        }
        room_state.name = data.room_name;
        room_state.participants = data.participants;
        room_state.ambiences = data.ambiences;
        room_state.ambience = data.ambience;
        room_state.chat = data.chat.history;
        started.value = true;
    } else if (data.event === "participants") {
        room_state.participants = data.participants;
    } else if (data.event === "ambience") {
        room_state.ambience = data.ambience;
    } else if (data.event === "chat") {
        room_state.chat.push(data.chat.history.pop());
    } else if (data.event === "error") {
        if (!is_error) {
            is_error = true;
            on_leave();
            window.alert(data.error);
            window.location = "/";
        }
    }
}

function on_audio_stream(data) {
    if (socket != null) {
        socket.send_message({type: "audio", data});
    }
}

function on_video_stream(data) {
    if (socket != null) {
        socket.send_message({type: "video", data});
    }
}
</script>

<template>
    <Create v-if="room_id.length < 2" @event_create="create"/>
    <Login v-else-if="!started" @event_login="start"/>
    <Room v-else/>
</template>

<style>
</style>
