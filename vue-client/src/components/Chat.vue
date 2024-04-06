<script setup>
import { ref } from 'vue';
import ChatMessage from './ChatMessage.vue';
import { room_state } from '../room_state';
import { event_bus } from '../event_bus';

const chat_message = ref("");
function on_chat() {
    event_bus.fire({type: "event_chat", data: chat_message.value});
    chat_message.value = "";
}
</script>

<template>
    <div class="cls_chat_cnt">
        <div class="cls_chat_title">
            <h3>Чат</h3>
        </div>
        <div class="cls_chat">
            <ChatMessage v-for="(chat_message) in room_state.chat" :key="chat_message.id" :chat_message="chat_message"/>
        </div>
        <form class="cls_chat_form" @submit.prevent="on_chat">
            <input class="cls_input" v-model="chat_message" placeholder="Ваше сообщение"/>
            <button class="cls_button" type="submit">Отправить</button>
        </form>
    </div>
</template>

<style>
    .cls_chat_cnt {
        display: grid; grid-template-rows: 2.5em 1fr 3.5em;
        border-right: 1px solid #ddd;
        transition: 0.2s;
    }
    .cls_chat_title {text-align: center;}
    .cls_chat {
        padding: 0.5em; display: flow; overflow: hidden;
        transition: 0.2s;
    }
    .cls_chat:hover {
        overflow: auto;
    }
    .cls_chat_form {
        padding: 0.5em;
        display: grid; grid-template-columns: 1fr 6em; gap: 0.5em;
        align-items: center;
        background-color: #f5f5dd;
    }
</style>