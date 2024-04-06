import { reactive } from 'vue'

export const room_state = reactive({
    ambiences: [],
    participants: [],
    chat: [],
    ambience: "",
    is_muted: true
})