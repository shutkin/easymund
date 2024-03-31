<script setup>
const props = defineProps(['chat_message'])
</script>
<template>
    <p>{{ chat_message.from }}: {{ chat_message.text }}</p>
</template>