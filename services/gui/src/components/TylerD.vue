<script setup lang="ts">
import Chat from '@/components/Chat.vue';
import type { TabsItem } from '@nuxt/ui';
import { useI18n } from 'vue-i18n';
import { computed } from 'vue';

const { t } = useI18n();

const items = computed<TabsItem[]>(() => [
    {
        label: t('tabs.ask'),
        icon: 'mdi-light:message',
        slot: 'ask',
    }, {
        label: t('tabs.ingest'),
        icon: 'mdi-light:plus-box',
        slot: 'ingest'
    }
])
</script>

<template>
  <div class="flex justify-center items-start min-h-screen bg-gray-50/50 dark:bg-gray-950 p-4 sm:p-8">
    <UContainer class="w-full max-w-4xl bg-white dark:bg-gray-900 shadow-sm border border-gray-200 dark:border-gray-800 rounded-xl overflow-hidden">
      <UTabs :items="items" class="w-full">
        <template #ask>
          <div class="p-4 min-h-[600px] flex flex-col">
            <Chat />
          </div>
        </template>
        <template #ingest>
          <div class="p-6 min-h-[600px]">
            <Ingest />
          </div>
        </template>
      </UTabs>
    </UContainer>
  </div>
</template>
