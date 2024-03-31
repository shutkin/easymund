<script setup>
import { ref } from 'vue';
import { NForm, NFormItem, NInput, NButton } from 'naive-ui'
const user_name = ref("");
defineEmits(['event_login'])
</script>

<template>
    <n-form @submit.prevent="$emit('event_login', user_name)">
        <n-form-item label="Представьтесь, пожалуйста">
            <n-input id="input_name" v-model="user_name" placeholder="Ваше имя"/>
        </n-form-item>
        <n-button :disabled="user_name.length < 3" type="submit">Старт</n-button>
    </n-form>
</template>