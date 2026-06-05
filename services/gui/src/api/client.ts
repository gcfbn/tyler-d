import { createClientFactory, createChannel, FetchTransport } from 'nice-grpc-web';
import type { CallOptions } from 'nice-grpc-web';
import { OrchestratorDefinition } from "@/generated/tyler_d";
import type { AskRequest, AskResponse, IngestRequest, IngestResponse } from "@/generated/tyler_d";

export interface ChatBotClient {
  ask(request: AskRequest, options?: CallOptions): Promise<AskResponse>;
  ingest(request: IngestRequest, options?: CallOptions): Promise<IngestResponse>;
}

const ORCHESTRATOR_URL = (import.meta.env?.VITE_ORCHESTRATOR_URL as string) || 'http://localhost:50052';

export function getChatBotClient(): ChatBotClient {
  const factory = createClientFactory();
  const channel = createChannel(ORCHESTRATOR_URL, FetchTransport());

  return factory.create(OrchestratorDefinition, channel) as unknown as ChatBotClient;
}
