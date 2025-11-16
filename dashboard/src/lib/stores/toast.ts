import {writable} from 'svelte/store'

export type ToastType = 'success' | 'error' | 'warning' | 'info'

export interface Toast {
    id: string
    type: ToastType
    message: string
    duration?: number
}

function createToastStore() {
    const {subscribe, update} = writable<Toast[]>([])

    return {
        subscribe,
        add: (type: ToastType, message: string, duration = 3000) => {
            const id = Math.random().toString(36).substring(7)
            const toast: Toast = {id, type, message, duration}

            update((toasts) => [...toasts, toast])

            if (duration > 0) {
                setTimeout(() => {
                    update((toasts) => toasts.filter((t) => t.id !== id))
                }, duration)
            }

            return id
        },
        remove: (id: string) => {
            update((toasts) => toasts.filter((t) => t.id !== id))
        },
        success: (message: string, duration?: number) => {
            return createToastStore().add('success', message, duration)
        },
        error: (message: string, duration?: number) => {
            return createToastStore().add('error', message, duration)
        },
        warning: (message: string, duration?: number) => {
            return createToastStore().add('warning', message, duration)
        },
        info: (message: string, duration?: number) => {
            return createToastStore().add('info', message, duration)
        },
    }
}

export const toasts = createToastStore()
