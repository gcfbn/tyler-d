import { getChatBotClient } from "@/api/client";
import { IngestRequest, type FileContent } from "@/generated/tyler_d";
import { defineStore } from "pinia";
import { ref } from "vue";

export const useIngestStore = defineStore('ingest', () => {
  const client = getChatBotClient();
  const ingested = ref<IngestRequest[]>([]);

  async function ingestMessage(text: string) {
    const ingestRequest = IngestRequest.create({ text });
    try {
      const result = await client.ingest(ingestRequest);
      
      if (!result.success) {
        throw new Error('Ingest failed');
      }

      ingested.value.push(ingestRequest);
    } catch (error) {
      console.error('Failed to ingest message', error);
      throw error;
    }
  }

  async function ingestFile(file: FileContent) {
    const ingestRequest = IngestRequest.create({ file });
    try {
      const result = await client.ingest(ingestRequest);
      
      if (!result.success) {
        throw new Error('Ingest failed');
      }
      
      ingested.value.push(ingestRequest);
    } catch (error) {
      console.error('Failed to ingest file', error);
      throw error;
    }
  }

  return { ingested, ingestMessage, ingestFile }
})
