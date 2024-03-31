<script setup>
import { ref } from 'vue';
const user_name = ref("");
defineEmits(['event_login'])
</script>

<template>
    <form @submit.prevent="$emit('event_login', user_name)">
        <label for="input_name">Представьтесь, пожалуйста</label>
        <input id="input_name" v-model="user_name" placeholder="Ваше имя"/>
        <div>
            <button :disabled="user_name.length < 3">Старт</button>
        </div>
    </form>
</template>