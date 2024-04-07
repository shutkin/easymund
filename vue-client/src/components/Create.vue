<script setup>
import { ref } from 'vue';
const conference_name = ref("");
defineEmits(['event_create'])
</script>

<template>
    <form class="cls_login_form" @submit.prevent="$emit('event_create', conference_name)">
        <span class="cls_login_label">Добро пожаловать в Easymund!</span>
        <div class="cls_login_input">
            <input class="cls_input" v-model="conference_name" placeholder="Название конференции"/>
            <button class="cls_button" :disabled="conference_name.length < 3" type="submit">Далее</button>
        </div>
    </form>
</template>

<style>
.cls_login_form {
    height: 6em; width: 35em; padding: 1.5em;
    display: flex; flex-direction: column; gap: 1em;
    background-color: #f1ead2;
    border-radius: 0.5em;
    box-shadow: 0 0.5em 1em rgba(0, 0, 0, 0.25);
}
.cls_login_label {
    text-align: center;
    color: #555;
}
.cls_login_input {
    padding: 1em;
    display: grid; grid-template-columns: 1fr 8em; gap: 0.5em;
}
</style>
