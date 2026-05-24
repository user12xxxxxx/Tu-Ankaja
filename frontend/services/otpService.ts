import type { OtpRecord, NumbersData, ParamsData, OtpHistoryData } from '@/types/otp';

const API_BASE = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001';

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

export async function generateOtp(): Promise<OtpRecord> {
  return apiFetch<OtpRecord>('/api/otp/generate', { method: 'POST' });
}

export async function getNumbers(): Promise<NumbersData> {
  return apiFetch<NumbersData>('/api/data/numbers');
}

export async function getParams(): Promise<ParamsData> {
  return apiFetch<ParamsData>('/api/data/params');
}

export async function getOtpHistory(): Promise<OtpRecord[]> {
  const data = await apiFetch<OtpHistoryData>('/api/otp/history');
  return data.history;
}
