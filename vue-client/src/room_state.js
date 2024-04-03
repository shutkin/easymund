import { reactive } from 'vue'
import { EventBus } from './event_bus'

export const room_state = reactive({
    ambiences: [],
    participants: [],
    chat: [],
    ambience: "",
    is_muted: true,
    event_bus: new EventBus()
})