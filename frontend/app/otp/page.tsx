'use client';

import { useCallback, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { KeyRound, Loader2, Copy, Check } from 'lucide-react';
import { useState } from 'react';
import { generateOtp, getOtpHistory } from '@/services/otpService';
import { useOtpStore } from '@/store/otpStore';

export default function OtpPage() {
  const currentOtp = useOtpStore((s) => s.currentOtp);
  const otpHistory = useOtpStore((s) => s.otpHistory);
  const isGenerating = useOtpStore((s) => s.isGenerating);
  const lastError = useOtpStore((s) => s.lastError);
  const setCurrentOtp = useOtpStore((s) => s.setCurrentOtp);
  const setOtpHistory = useOtpStore((s) => s.setOtpHistory);
  const setGenerating = useOtpStore((s) => s.setGenerating);
  const setError = useOtpStore((s) => s.setError);

  const [copied, setCopied] = useState(false);

  const handleGenerate = useCallback(async () => {
    setGenerating(true);
    setError(undefined);
    try {
      const otp = await generateOtp();
      setCurrentOtp(otp);
      const history = await getOtpHistory();
      setOtpHistory(history);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to generate OTP');
    } finally {
      setGenerating(false);
    }
  }, [setGenerating, setError, setCurrentOtp, setOtpHistory]);

  const handleCopy = useCallback(() => {
    if (!currentOtp) return;
    navigator.clipboard.writeText(currentOtp.otp);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  }, [currentOtp]);

  useEffect(() => {
    getOtpHistory().then(setOtpHistory).catch(() => {});
    const timer = setInterval(() => {
      getOtpHistory().then(setOtpHistory).catch(() => {});
    }, 2000);
    return () => clearInterval(timer);
  }, [setOtpHistory]);

  return (
    <div className="mx-auto max-w-lg px-4 py-8">
      <div className="mb-8 text-center">
        <h1 className="text-2xl font-semibold text-neutral-100">OTP Generator</h1>
        <p className="mt-1 text-sm text-neutral-500">
          Generate one-time passwords from hardware entropy via MQTT
        </p>
      </div>

      <div className="mb-6 rounded-xl border border-white/[0.07] bg-white/[0.03] p-6 text-center backdrop-blur-xl">
        <div className="mb-4 min-h-[72px] flex items-center justify-center">
          {currentOtp ? (
            <motion.div
              key={currentOtp.otp}
              initial={{ opacity: 0, scale: 0.9 }}
              animate={{ opacity: 1, scale: 1 }}
              className="flex flex-col items-center gap-2"
            >
              <p className="font-mono text-5xl font-bold tracking-[0.3em] text-emerald-300">
                {currentOtp.otp}
              </p>
              <div className="flex items-center gap-3 text-xs text-neutral-500">
                <span>Source: {String(currentOtp.source_number).padStart(4, '0')}</span>
                <span className="text-neutral-700">|</span>
                <span>{currentOtp.created_at}</span>
              </div>
            </motion.div>
          ) : (
            <p className="text-sm text-neutral-600">No OTP generated yet</p>
          )}
        </div>

        <div className="flex justify-center gap-2">
          <button
            onClick={handleGenerate}
            disabled={isGenerating}
            className="flex items-center gap-2 rounded-lg bg-emerald-500 px-5 py-2.5 text-sm font-medium text-black transition-colors hover:bg-emerald-400 disabled:opacity-50"
          >
            {isGenerating ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <KeyRound className="h-4 w-4" />
            )}
            Generate OTP
          </button>

          {currentOtp && (
            <button
              onClick={handleCopy}
              className="flex items-center gap-1.5 rounded-lg border border-white/[0.07] bg-white/[0.03] px-3 py-2.5 text-sm text-neutral-300 transition-colors hover:bg-white/[0.06]"
            >
              {copied ? <Check className="h-3.5 w-3.5 text-emerald-400" /> : <Copy className="h-3.5 w-3.5" />}
              {copied ? 'Copied' : 'Copy'}
            </button>
          )}
        </div>

        <AnimatePresence>
          {lastError && (
            <motion.p
              initial={{ opacity: 0, y: 4 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -4 }}
              className="mt-3 rounded-md border border-rose-300/20 bg-rose-400/10 px-2 py-1.5 text-xs text-rose-200"
            >
              {lastError}
            </motion.p>
          )}
        </AnimatePresence>
      </div>

      <div>
        <div className="mb-2 flex items-center justify-between">
          <p className="text-[11px] uppercase tracking-wider text-neutral-500">History</p>
          <span className="text-[10px] text-neutral-600">{otpHistory.length} generated</span>
        </div>

        <div className="space-y-1.5">
          <AnimatePresence mode="popLayout">
            {otpHistory.length > 0 ? (
              otpHistory.slice(0, 20).map((record, i) => (
                <motion.div
                  key={`${record.otp}-${record.created_at}-${i}`}
                  layout
                  initial={{ opacity: 0, y: 8 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -8 }}
                  className="flex items-center justify-between rounded-lg border border-white/[0.05] bg-white/[0.02] px-3 py-2"
                >
                  <span className="font-mono text-sm font-medium text-neutral-200 tracking-wider">
                    {record.otp}
                  </span>
                  <div className="flex items-center gap-3 text-[10px] text-neutral-600">
                    <span>src: {String(record.source_number).padStart(4, '0')}</span>
                    <span>{record.created_at}</span>
                  </div>
                </motion.div>
              ))
            ) : (
              <div className="flex h-20 items-center justify-center rounded-lg border border-dashed border-white/[0.07] bg-white/[0.015] text-xs text-neutral-600">
                Generate an OTP to see history
              </div>
            )}
          </AnimatePresence>
        </div>
      </div>
    </div>
  );
}
