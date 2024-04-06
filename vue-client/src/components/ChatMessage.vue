<script setup>
const props = defineProps(['chat_message'])
</script>
<template>
    <div class="cls_chat_message">
        <div class="cls_chat_msg_author">{{ chat_message.from }}</div>
        <div class="cls_chat_msg_body">{{ chat_message.text }}</div>
        <div class="cls_chat_msg_time">{{ chat_message.time }}</div>
    </div>
</template>
<style>
    .cls_chat_message {
        height: fit-content;
        display: block;
        background-color: #add8e740;
        border-radius: 0.5em;
        border: 1px solid #add8e780;
        padding: 0.2em;
        margin-bottom: 0.5em;
    }
    .cls_chat_msg_author {
        font-size: small;
        font-weight: bold;
        color: #365194;
    }
    .cls_chat_msg_body {
        height: fit-content;
        padding: 0.3em;
    }
    .cls_chat_msg_time {
        font-size: small;
        text-align: right;
        color: rgba(0, 0, 0, 0.5);
    }
</style>