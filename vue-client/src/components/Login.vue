<script setup>
import { ref } from 'vue';
const user_name = ref("");
</script>

<template>
    <div class="wrapper">
        <p>Представьтесь, пожалуйста</p>
        <input v-model="user_name" placeholder="Ваше имя">
        <button v-if="user_name.length > 2" @click="$emit('event_login', user_name)">Старт</button>
    </div>
</template>