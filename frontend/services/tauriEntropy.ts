import type {
  EntropyStats,
  GeneratedMaterialKind,
  GeneratedOutput,
  GenerationOptions,
  IntegritySnapshot,
  IntegrityStatus,
  RealtimeEntropySample,
  SecurityEvent
} from '@/types/entropy';

const ENTROPY_EVENTS = ['entropy_update', 'entropy:update'] as const;

/// HTTP API base URL for the Rust engine when running outside Tauri.
const API_BASE = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001';

type UnknownRecord = Record<string, unknown>;
type Unlisten = () => void;
let outputIdSequence = 0;

export function isTauriRuntime() {
  if (typeof window === 'undefined') {
    return false;
  }

  const runtime = window as unknown as UnknownRecord;
  return Boolean(runtime.__TAURI_INTERNALS__ || runtime.__TAURI__);
}

// ── HTTP API helpers ────────────────────────────────────────────────

async function apiFetch<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    ...options,
    headers: { 'Content-Type': 'application/json', ...options?.headers }
  });

  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(body.error || `API error: ${res.status}`);
  }

  return res.json();
}

// ── Public API ──────────────────────────────────────────────────────

export async function getEntropyIntegrity(): Promise<IntegritySnapshot> {
  if (isTauriRuntime()) {
    const { invoke } = await import('@tauri-apps/api/core');
    const payload = await invoke<unknown>('get_entropy_integrity');
    return normalizeIntegrity(payload);
  }

  try {
    const payload = await apiFetch<unknown>('/api/integrity');
    return normalizeIntegrity(payload);
  } catch {
    return {
      status: 'offline',
      label: 'Engine offline',
      checkedAt: new Date().toISOString()
    };
  }
}

export async function getEntropyStats(): Promise<EntropyStats | null> {
  if (isTauriRuntime()) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const payload = await invoke<unknown>('get_entropy_stats');
      return normalizeStats(payload);
    } catch {
      return null;
    }
  }

  try {
    const payload = await apiFetch<unknown>('/api/stats');
    return normalizeStats(payload);
  } catch {
    return null;
  }
}

export async function getSecurityEvents(): Promise<SecurityEvent[]> {
  if (isTauriRuntime()) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const payload = await invoke<unknown>('drain_security_events');
      return normalizeSecurityEvents(payload);
    } catch {
      return [];
    }
  }

  try {
    const payload = await apiFetch<unknown>('/api/events');
    return normalizeSecurityEvents(payload);
  } catch {
    return [];
  }
}

export async function listenForEntropyUpdates(
  onUpdate: (sample: RealtimeEntropySample) => void
): Promise<Unlisten> {
  if (isTauriRuntime()) {
    const { listen } = await import('@tauri-apps/api/event');
    const unlisteners = await Promise.all(
      ENTROPY_EVENTS.map((eventName) =>
        listen<unknown>(eventName, (event) => {
          onUpdate(normalizeEntropySample(event.payload));
        })
      )
    );
    return () => { for (const u of unlisteners) u(); };
  }

  // In HTTP mode, poll stats and synthesize entropy samples.
  const timer = setInterval(async () => {
    try {
      const stats = await apiFetch<EntropyStats>('/api/stats');
      onUpdate({
        intensity: Math.min(1, stats.pool_entropy_bits / 256),
        stability: stats.pool_well_seeded ? 1 : stats.pool_entropy_bits / 512,
        timestamp: new Date().toISOString()
      });
    } catch {
      // Engine not reachable — skip this tick.
    }
  }, 2000);

  return () => clearInterval(timer);
}

export async function generateEntropyMaterial(
  kind: GeneratedMaterialKind,
  options: GenerationOptions,
  latestBackendFingerprint?: string
): Promise<GeneratedOutput> {
  if (isTauriRuntime()) {
    const { invoke } = await import('@tauri-apps/api/core');
    const payload = await invoke<unknown>(commandForKind(kind), argsForKind(kind, options));
    return normalizeGeneratedOutput(kind, payload, latestBackendFingerprint);
  }

  // HTTP API path for each kind.
  const apiPath = kind === 'aes-key'
    ? '/api/generate/aes-key'
    : kind === 'password'
      ? '/api/generate/password'
      : '/api/generate/session-token';

  const body = kind === 'password'
    ? JSON.stringify({ length: clamp(Math.round(options.length ?? 24), 12, 96) })
    : '{}';

  const payload = await apiFetch<unknown>(apiPath, { method: 'POST', body });
  return normalizeGeneratedOutput(kind, payload, latestBackendFingerprint);
}

function commandForKind(kind: GeneratedMaterialKind) {
  if (kind === 'aes-key') {
    return 'generate_aes_key';
  }

  if (kind === 'password') {
    return 'generate_password';
  }

  return 'generate_session_token';
}

function argsForKind(kind: GeneratedMaterialKind, options: GenerationOptions) {
  if (kind !== 'password') {
    return undefined;
  }

  return {
    length: clamp(Math.round(options.length ?? 24), 12, 96)
  };
}

function normalizeEntropySample(payload: unknown): RealtimeEntropySample {
  const record = asRecord(payload);

  return {
    intensity: normalizeUnit(readNumber(record, 'intensity', 'entropyIntensity', 'entropy_intensity'), 0),
    stability: normalizeUnit(readNumber(record, 'stability', 'entropyStability', 'entropy_stability'), 0),
    fingerprint: readString(record, 'fingerprint', 'entropyFingerprint', 'entropy_fingerprint'),
    sequence: readNumber(record, 'sequence', 'seq'),
    timestamp: readString(record, 'timestamp', 'createdAt', 'created_at') ?? new Date().toISOString()
  };
}

function normalizeIntegrity(payload: unknown): IntegritySnapshot {
  if (typeof payload === 'string') {
    return {
      status: normalizeIntegrityStatus(payload),
      label: labelForStatus(normalizeIntegrityStatus(payload)),
      checkedAt: new Date().toISOString()
    };
  }

  const record = asRecord(payload);
  const rawStatus = readString(record, 'status', 'state', 'integrity');
  const status = normalizeIntegrityStatus(rawStatus);

  return {
    status,
    label: readString(record, 'label', 'message') ?? labelForStatus(status),
    score: normalizeUnit(readNumber(record, 'score', 'confidence'), undefined),
    fingerprint: readString(record, 'fingerprint', 'entropyFingerprint', 'entropy_fingerprint'),
    checkedAt: readString(record, 'checkedAt', 'checked_at', 'timestamp') ?? new Date().toISOString()
  };
}

function normalizeStats(payload: unknown): EntropyStats | null {
  const record = asRecord(payload);
  if (!record || Object.keys(record).length === 0) return null;

  const sourceQuality = Array.isArray(record.source_quality)
    ? (record.source_quality as unknown[]).map((sq) => {
        const s = asRecord(sq);
        return {
          source_id: readNumber(s, 'source_id') ?? 0,
          tier: readString(s, 'tier') ?? 'unknown',
          min_entropy_bits_per_byte: readNumber(s, 'min_entropy_bits_per_byte') ?? 0,
          confidence: readNumber(s, 'confidence') ?? 0,
          observations: readNumber(s, 'observations') ?? 0,
          total_bytes: readNumber(s, 'total_bytes') ?? 0
        };
      })
    : [];

  return {
    pool_fills: readNumber(record, 'pool_fills') ?? 0,
    bytes_generated: readNumber(record, 'bytes_generated') ?? 0,
    reseed_count: readNumber(record, 'reseed_count') ?? 0,
    health_status: readString(record, 'health_status') ?? 'unknown',
    drbg_bytes_since_reseed: readNumber(record, 'drbg_bytes_since_reseed') ?? 0,
    pool_entropy_bits: readNumber(record, 'pool_entropy_bits') ?? 0,
    pool_well_seeded: Boolean(record.pool_well_seeded),
    source_quality: sourceQuality
  };
}

function normalizeSecurityEvents(payload: unknown): SecurityEvent[] {
  if (!Array.isArray(payload)) return [];

  return payload.map((item) => {
    if (typeof item === 'string') {
      return { kind: 'event', detail: item, timestamp: new Date().toISOString() };
    }
    const record = asRecord(item);
    return {
      kind: readString(record, 'kind', 'type', 'event_type') ?? 'event',
      detail: readString(record, 'detail', 'message', 'description') ?? JSON.stringify(item),
      timestamp: readString(record, 'timestamp', 'created_at') ?? new Date().toISOString()
    };
  });
}

function normalizeGeneratedOutput(
  kind: GeneratedMaterialKind,
  payload: unknown,
  latestBackendFingerprint?: string
): GeneratedOutput {
  const record = asRecord(payload);
  const value =
    typeof payload === 'string'
      ? payload
      : readString(record, 'value', 'generatedValue', 'generated_value', 'key', 'aesKey', 'aes_key', 'password', 'token', 'sessionToken', 'session_token');

  if (!value) {
    throw new Error('The entropy engine returned an empty generated value.');
  }

  const timestamp = readString(record, 'timestamp', 'createdAt', 'created_at') ?? new Date().toISOString();

  return {
    id: readString(record, 'id', 'requestId', 'request_id') ?? createUiId(kind, timestamp),
    kind,
    value,
    timestamp,
    entropyFingerprint:
      readString(record, 'entropyFingerprint', 'entropy_fingerprint', 'fingerprint') ?? latestBackendFingerprint
  };
}

function normalizeIntegrityStatus(value: unknown): IntegrityStatus {
  const normalized = String(value ?? '').toLowerCase();

  if (['verified', 'healthy', 'secure', 'ok', 'valid'].includes(normalized)) {
    return 'verified';
  }

  if (['degraded', 'warning', 'unstable', 'watch'].includes(normalized)) {
    return 'degraded';
  }

  if (['compromised', 'failed', 'critical', 'invalid'].includes(normalized)) {
    return 'compromised';
  }

  if (['offline', 'disconnected', 'unavailable'].includes(normalized)) {
    return 'offline';
  }

  return 'unknown';
}

function labelForStatus(status: IntegrityStatus) {
  const labels: Record<IntegrityStatus, string> = {
    verified: 'Integrity verified',
    degraded: 'Integrity degraded',
    compromised: 'Integrity compromised',
    offline: 'Engine offline',
    unknown: 'Integrity pending'
  };

  return labels[status];
}

function asRecord(value: unknown): UnknownRecord {
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    return value as UnknownRecord;
  }

  return {};
}

function readString(record: UnknownRecord, ...keys: string[]) {
  for (const key of keys) {
    const value = record[key];

    if (typeof value === 'string' && value.length > 0) {
      return value;
    }
  }

  return undefined;
}

function readNumber(record: UnknownRecord, ...keys: string[]) {
  for (const key of keys) {
    const value = record[key];

    if (typeof value === 'number' && Number.isFinite(value)) {
      return value;
    }
  }

  return undefined;
}

function normalizeUnit(value: number | undefined, fallback: number): number;
function normalizeUnit(value: number | undefined, fallback: undefined): number | undefined;
function normalizeUnit(value: number | undefined, fallback: number | undefined) {
  if (value === undefined) {
    return fallback;
  }

  return clamp(value > 1 ? value / 100 : value, 0, 1);
}

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}

function createUiId(kind: GeneratedMaterialKind, timestamp: string) {
  outputIdSequence += 1;
  return `${kind}-${timestamp}-${outputIdSequence}`;
}
