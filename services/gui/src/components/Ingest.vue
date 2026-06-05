<script setup lang="ts">
import { useIngestStore } from '@/stores/ingest';
import type { TabsItem } from '@nuxt/ui';
import { ref, type Ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';

const ingestStore = useIngestStore();
const toast = useToast();
const { t } = useI18n();

const items = computed<TabsItem[]>(() => [
    {
        label: t('ingest.tabs.file'),
        icon: 'mdi-light:file-plus',
        slot: 'file',
        'value': 'file'
    }, {
        label: t('ingest.tabs.text'),
        icon: 'mdi-light:note-text',
        slot: 'text',
        value: 'text'
    }
])

const activeTab: Ref<'file' | 'text'> = ref('file')
const inputFile: Ref<File | null> = ref(null)
const inputText = ref('')

const isButtonLoading = ref(false)

const handleFileSubmit = async () => {
    if (!inputFile.value) {
        return;
    }
    const fullFileName = inputFile.value.name;

    const request = {
        content: await inputFile.value.bytes(),
        fileName: inputFile.value.name,
        fileType: fullFileName.split('.').at(-1) || fullFileName
    };

    try {
        await ingestStore.ingestFile(request);
        inputFile.value = null
        toast.add({
            color: 'success',
            title: t('ingest.notifications.success.title'),
            description: t('ingest.notifications.success.file')
        })
    } catch {
        toast.add({
            color: 'error',
            title: t('ingest.notifications.error.title'),
            description: t('ingest.notifications.error.file')
        })
    } finally {
        isButtonLoading.value = false
    }
}

const handleTextSubmit = async () => {
    if (!inputText.value) {
        return;
    }

    try {
        await ingestStore.ingestMessage(inputText.value);
        inputText.value = '';
        toast.add({
            color: 'success',
            title: t('ingest.notifications.success.title'),
            description: t('ingest.notifications.success.text')
        })
    } catch {
        toast.add({
            color: 'error',
            title: t('ingest.notifications.error.title'),
            description: t('ingest.notifications.error.text')
        })
    } finally {
        isButtonLoading.value = false
    }
}

const onSubmit = async () => {
    isButtonLoading.value = true
    switch(activeTab.value) {
        case 'file': await handleFileSubmit(); break;
        case 'text': await handleTextSubmit(); break;
        default: console.error('Unreachable case statement - should never happen.')
    }
}
</script>

<template>
  <div class="flex flex-col gap-8 max-w-2xl mx-auto">
    <UTabs :items="items" v-model="activeTab" class="w-full">
      <template #file>
        <div class="py-6">
          <UFileUpload 
            v-model="inputFile" 
            class="w-full min-h-64 border-2 border-dashed border-gray-200 dark:border-gray-800 rounded-xl flex items-center justify-center bg-gray-50/30 dark:bg-gray-900/30" 
            accept="application/pdf" 
          />
        </div>
      </template>
      <template #text>
        <div class="py-6">
          <UTextarea 
            v-model="inputText" 
            :placeholder="t('ingest.placeholders.textarea')" 
            :maxrows="16" 
            autoresize
            class="w-full"
          />
        </div>
      </template>
    </UTabs>
    
    <div class="flex justify-end">
      <UButton 
        :label="t('ingest.buttons.submit')" 
        icon="mdi-light:send"
        variant="solid" 
        size="lg"
        :loading="isButtonLoading" 
        class="px-8 shadow-md justify-center"
        @click="onSubmit"
      />
    </div>
  </div>
</template>
