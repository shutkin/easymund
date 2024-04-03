<script setup>
const props = defineProps(['chat_message'])
</script>
<template>
    <div class="cls_chat_message">
        <span>{{ chat_message.from }}: {{ chat_message.text }}</span>
    </div>
</template>
<style>
    .cls_chat_message {height: fit-content; padding: 0.5em;}
</style>