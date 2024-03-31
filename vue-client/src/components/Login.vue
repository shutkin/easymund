<script setup>
import { ref } from 'vue';
const user_name = ref("");
defineEmits(['event_login'])
</script>

<template>
    <VaForm @submit.prevent="$emit('event_login', user_name)" class="flex flex-col items-baseline gap-6" tag="form">
        <VaInput v-model="user_name" label="Ваше имя"/>
        <VaButton type="submit" :disabled="user_name.length < 3">Старт</VaButton>
    </VaForm>
</template>