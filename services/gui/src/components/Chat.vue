<script setup lang="ts">
import { useMessagesStore } from '@/stores/messages'
import { storeToRefs } from 'pinia'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

const toast = useToast()
const { t } = useI18n()

const store = useMessagesStore()
const { status, messages } = storeToRefs(store)
const { sendChatMessage, cancelLastMessage } = store

const chatStatus = computed(() => (status.value === 'idle' ? 'ready' : 'submitted'))

const chatMessages = computed(() =>
  messages.value.map((message, idx) => ({
    id: idx.toString(),
    role: message.role,
    parts: [
      {
        type: 'text',
        text: message.content
      }
    ]
  }))
)

const userInput = ref('')
const onSubmit = async () => {
  if (!userInput.value) return

  const query = userInput.value
  userInput.value = ''

  try {
    await sendChatMessage(query)
  } catch (error) {
    console.warn('Error posting the message', error)
    toast.add({
      title: t('chat.notifications.error.title'),
      description: t('chat.notifications.error.description'),
      color: 'error'
    })
  }
}

const onStop = () => {
  cancelLastMessage()
}
</script>

<template>
  <UChatPalette class="flex-1 flex flex-col h-full overflow-hidden">
    <UChatMessages
      :status="chatStatus"
      :user="{
        side: 'right',
        variant: 'soft'
      }"
      :assistant="{
        side: 'left',
        variant: 'outline'
      }"
      :messages="chatMessages"
      class="flex-1 overflow-y-auto px-2"
    />
    <template #prompt>
      <div class="border-t border-gray-100 dark:border-gray-800 p-4 pt-6">
        <UChatPrompt
          v-model="userInput"
          @submit="onSubmit"
          :submitOnEnter="true"
          :maxrows="12"
          autoresize
          class="shadow-sm"
        >
          <UChatPromptSubmit :status="chatStatus" @stop="onStop" />
        </UChatPrompt>
      </div>
    </template>
  </UChatPalette>
</template>
