import { create } from 'zustand';
import type {
  EntropyRuntime,
  EntropyStats,
  GeneratedMaterialKind,
  GeneratedOutput,
  IntegritySnapshot,
  RealtimeEntropySample,
  SecurityEvent
} from '@/types/entropy';

type GenerationState = Record<GeneratedMaterialKind, boolean>;

type EntropyState = {
  runtime: EntropyRuntime;
  entropy: RealtimeEntropySample;
  hasEntropySignal: boolean;
  integrity: IntegritySnapshot;
  outputs: GeneratedOutput[];
  generating: GenerationState;
  stats: EntropyStats | null;
  securityEvents: SecurityEvent[];
  lastError?: string;
  setRuntime: (runtime: EntropyRuntime) => void;
  setIntegrity: (integrity: IntegritySnapshot) => void;
  pushEntropySample: (sample: RealtimeEntropySample) => void;
  addOutput: (output: GeneratedOutput) => void;
  setGenerating: (kind: GeneratedMaterialKind, generating: boolean) => void;
  setStats: (stats: EntropyStats) => void;
  pushSecurityEvents: (events: SecurityEvent[]) => void;
  setError: (message?: string) => void;
};

const initialEntropy: RealtimeEntropySample = {
  intensity: 0,
  stability: 0,
  timestamp: ''
};

const initialGenerationState: GenerationState = {
  'aes-key': false,
  password: false,
  'session-token': false
};

export const useEntropyStore = create<EntropyState>((set) => ({
  runtime: 'unknown',
  entropy: initialEntropy,
  hasEntropySignal: false,
  integrity: {
    status: 'unknown',
    label: 'Integrity pending'
  },
  outputs: [],
  generating: initialGenerationState,
  stats: null,
  securityEvents: [],
  setRuntime: (runtime) => set({ runtime }),
  setIntegrity: (integrity) =>
    set((state) => ({
      integrity: {
        ...state.integrity,
        ...integrity
      }
    })),
  pushEntropySample: (sample) =>
    set((state) => ({
      entropy: {
        ...state.entropy,
        ...sample,
        intensity: clamp01(sample.intensity),
        stability: clamp01(sample.stability),
        timestamp: sample.timestamp || new Date().toISOString()
      },
      hasEntropySignal: true,
      lastError: undefined
    })),
  addOutput: (output) =>
    set((state) => ({
      outputs: [output, ...state.outputs].slice(0, 6),
      lastError: undefined
    })),
  setGenerating: (kind, generating) =>
    set((state) => ({
      generating: {
        ...state.generating,
        [kind]: generating
      }
    })),
  setStats: (stats) => set({ stats }),
  pushSecurityEvents: (events) =>
    set((state) => ({
      securityEvents: [...events, ...state.securityEvents].slice(0, 20)
    })),
  setError: (message) => set({ lastError: message })
}));

function clamp01(value: number) {
  return Math.min(1, Math.max(0, value));
}
