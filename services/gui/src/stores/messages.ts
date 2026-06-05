import { getChatBotClient } from '@/api/client'
import { AskRequest, type Message } from '@/generated/tyler_d'
import { defineStore } from 'pinia'
import { ref } from 'vue'

export type StoreStatus = 'idle' | 'loading'

export const useMessagesStore = defineStore('messages', () => {
  const client = getChatBotClient()
  const messages = ref<Message[]>([])
  const status = ref<StoreStatus>('idle')
  
  let abortController: AbortController | null = null

  async function sendChatMessage(query: string) {
    status.value = 'loading'
    
    const userMessage: Message = { role: 'user', content: query }
    messages.value.push(userMessage)

    abortController = new AbortController()

    try {
      const chatResponse = await client.ask(
        AskRequest.create({ history: messages.value, query }),
        { signal: abortController.signal }
      )

      if (chatResponse.answer) {
        messages.value.push({ role: 'assistant', content: chatResponse.answer })
      }
    } catch (error: any) {
      // Ignore errors caused by manual cancellation
      if (error.name === 'AbortError' || error.code === 'CANCELLED') {
        return
      }
      
      throw error
    } finally {
      status.value = 'idle'
      abortController = null
    }
  }

  function cancelLastMessage() {
    // Terminate the active gRPC request
    if (abortController) {
      abortController.abort()
      abortController = null
    }

    // Rollback the UI by removing the pending user message
    const lastUserIdx = messages.value.findLastIndex((m) => m.role === 'user')
    if (lastUserIdx !== -1) {
      messages.value.splice(lastUserIdx, 1)
    }

    status.value = 'idle'
  }

  return {
    messages,
    status,
    sendChatMessage,
    cancelLastMessage
  }
})
