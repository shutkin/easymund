import { reactive } from 'vue'

export const room_state = reactive({
    ambiences: [],
    participants: [],
    ambience: "",
    is_muted: true,
})