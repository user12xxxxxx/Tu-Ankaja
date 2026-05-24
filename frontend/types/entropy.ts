export type IntegrityStatus = 'verified' | 'degraded' | 'compromised' | 'offline' | 'unknown';

export type EntropyRuntime = 'unknown' | 'tauri' | 'browser';

export type RealtimeEntropySample = {
  intensity: number;
  stability: number;
  fingerprint?: string;
  sequence?: number;
  timestamp: string;
};

export type IntegritySnapshot = {
  status: IntegrityStatus;
  label: string;
  score?: number;
  fingerprint?: string;
  checkedAt?: string;
};

export type GeneratedMaterialKind = 'aes-key' | 'password' | 'session-token';

export type GeneratedOutput = {
  id: string;
  kind: GeneratedMaterialKind;
  value: string;
  timestamp: string;
  entropyFingerprint?: string;
};

export type GenerationOptions = {
  length?: number;
};

export type SourceQualitySummary = {
  source_id: number;
  tier: string;
  min_entropy_bits_per_byte: number;
  confidence: number;
  observations: number;
  total_bytes: number;
};

export type EntropyStats = {
  pool_fills: number;
  bytes_generated: number;
  reseed_count: number;
  health_status: string;
  drbg_bytes_since_reseed: number;
  pool_entropy_bits: number;
  pool_well_seeded: boolean;
  source_quality: SourceQualitySummary[];
};

export type SecurityEvent = {
  kind: string;
  detail: string;
  timestamp: string;
};
