import { create } from 'zustand';
import type { OtpRecord } from '@/types/otp';

type OtpState = {
  currentOtp: OtpRecord | null;
  otpHistory: OtpRecord[];
  numbers: number[];
  params: string[];
  isGenerating: boolean;
  lastError?: string;

  setCurrentOtp: (otp: OtpRecord) => void;
  setOtpHistory: (history: OtpRecord[]) => void;
  setNumbers: (numbers: number[]) => void;
  setParams: (params: string[]) => void;
  setGenerating: (v: boolean) => void;
  setError: (msg?: string) => void;
};

export const useOtpStore = create<OtpState>((set) => ({
  currentOtp: null,
  otpHistory: [],
  numbers: [],
  params: [],
  isGenerating: false,
  lastError: undefined,

  setCurrentOtp: (otp) => set({ currentOtp: otp }),
  setOtpHistory: (history) => set({ otpHistory: history }),
  setNumbers: (numbers) => set({ numbers }),
  setParams: (params) => set({ params }),
  setGenerating: (v) => set({ isGenerating: v }),
  setError: (msg) => set({ lastError: msg }),
}));
