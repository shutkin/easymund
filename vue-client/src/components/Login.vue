<script setup>
import { ref } from 'vue';
const user_name = ref("");
defineEmits(['event_login'])
</script>

<template>
    <div class="wrapper">
        <form @submit.prevent="$emit('event_login', user_name)">
            <label for="input_name">Представьтесь, пожалуйста</label>
            <br>
            <input id="input_name" v-model="user_name" placeholder="Ваше имя"/>
            <button :disabled="user_name.length < 3">Старт</button>
        </form>
    </div>
</template>