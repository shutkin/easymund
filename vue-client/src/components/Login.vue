<script setup>
import { ref } from 'vue';
const user_name = ref("");
defineEmits(['event_login'])
</script>

<template>
    <form class="cls_login_form" @submit.prevent="$emit('event_login', user_name)">
        <span class="cls_login_label">Представьтесь, пожалуйста</span>
        <div class="cls_login_input">
            <input class="cls_input" v-model="user_name" placeholder="Ваше имя"/>
            <button class="cls_button" :disabled="user_name.length < 3" type="submit">Старт</button>
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
.cls_input {
    padding: 0.5em;
    border: 1px solid #ddd;
    border-radius: 0.5em;
}
.cls_input:focus {
    box-shadow: 0 0.25em 0.5em rgba(0, 0, 0, 0.2);
    outline: none;
}
.cls_button {
    padding: 0.5em;
    border: none;
    border-radius: 0.5em;
    background-color: #365194;
    color: #fff;
    font-weight: bold;
    text-align: center;
    cursor: pointer;
    transition: background-color 0.25s;
}
.cls_button:hover {
    background-color: #213f84;
    box-shadow: 0 0.25em 0.5em rgba(0, 0, 0, 0.2);
}
</style>
