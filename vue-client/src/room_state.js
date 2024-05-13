import { reactive } from 'vue'

export const room_state = reactive({
    name: "",
    ambiences: [],
    participants: [],
    chat: [],
    ambience: "",
    self_id: 0,
    is_admin: false,
    is_muted: true,
    is_screen_sharing: false,
    is_talking: []
})