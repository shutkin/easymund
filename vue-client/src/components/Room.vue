<script setup>
import { ref, watch } from 'vue';
import Participant from './Participant.vue';
import ChatMessage from './ChatMessage.vue';
import { room_state } from '../room_state';

const emit = defineEmits(['event_leave', 'event_ambience', 'event_mic', 'event_chat']);

const ambience_select = ref("");
watch(() => room_state.ambience, (ambience_id) => {
    for (const ambience of room_state.ambiences) {
        if (ambience.id === ambience_id && ambience_select.value != ambience.name) {
            console.log("change ambience to " + ambience.name);
            ambience_select.value = ambience.name;
        }
    }
});
const mic_state = ref(room_state.is_muted ? "🎤 выкл" : "🎤 вкл");
watch(() => room_state.is_muted, (is_muted) => {
    mic_state.value = is_muted ? "🎤 выкл" : "🎤 вкл";
});

const chat_message = ref("");
function on_chat() {
    emit('event_chat', chat_message.value);
    chat_message.value = "";
}
</script>
<template>
    <section class="cls_layout">
        <div class="cls_chat">
            <h3>Чат</h3>
            <ChatMessage v-for="(chat_message) in room_state.chat" :key="chat_message.id" :chat_message="chat_message"/>
        </div>
        <div class="cls_main">
            <h3>Участники:</h3>
            <div class="cls_participants">
                <Participant v-for="(participant) in room_state.participants" :key="participant.id" :participant="participant"/>
            </div>
        </div>
        <div class="cls_controls">
            <form class="cls_chat_form" @submit.prevent="on_chat">
                <input id="input_chat" v-model="chat_message"/>
                <button type="submit">Отправить</button>
            </form>
            <button id="button_mic" @click="$emit('event_mic')">{{mic_state}}</button>
            <div>
                <span>Фоновый звук:</span>
                <select v-model="ambience_select" @change="$emit('event_ambience', ambience_select)">
                    <option v-for="(ambience) in room_state.ambiences" :key="ambience.id" :id="ambience.id">{{ambience.name}}</option>
                </select>
            </div>
            <button @click="$emit('event_leave')">Выйти</button>
        </div>
    </section>
</template>

<style>
.cls_layout {
    width: 100%;
    height: 100%;

    display: grid;
    grid:
        "left right" 1fr
        "bottom bottom" auto;
    gap: 1em;
}
.cls_chat {
    grid-area: left;
}
.cls_main {
    grid-area: right;
}
.cls_controls {
    grid-area: bottom;
    display: flex;
    gap: 1em;
}
.cls_participants {
    display: flex;
}
.cls_chat_form {
    border: solid 1px;
}
</style>