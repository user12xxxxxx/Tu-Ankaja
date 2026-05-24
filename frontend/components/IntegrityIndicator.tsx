'use client';

import { motion } from 'framer-motion';
import { ShieldCheck, ShieldAlert, ShieldOff, Radio } from 'lucide-react';
import { premiumEase, timing } from '@/animations/timing';
import { cn } from '@/lib/utils';
import { useEntropyStore } from '@/store/entropyStore';
import type { IntegrityStatus } from '@/types/entropy';

const statusStyles: Record<IntegrityStatus, string> = {
  verified: 'border-emerald-300/30 bg-emerald-300/10 text-emerald-100',
  degraded: 'border-amber-300/30 bg-amber-300/10 text-amber-100',
  compromised: 'border-rose-300/35 bg-rose-400/10 text-rose-100',
  offline: 'border-white/10 bg-white/[0.045] text-neutral-300',
  unknown: 'border-cyan-200/20 bg-cyan-200/10 text-cyan-100'
};

export function IntegrityIndicator() {
  const integrity = useEntropyStore((state) => state.integrity);
  const entropy = useEntropyStore((state) => state.entropy);
  const runtime = useEntropyStore((state) => state.runtime);
  const Icon = iconForStatus(integrity.status);
  const fingerprint = integrity.fingerprint ?? entropy.fingerprint;

  return (
    <motion.div
      animate={{ opacity: 1, y: 0 }}
      className={cn(
        'inline-flex min-h-10 items-center gap-3 rounded-lg border px-3 py-2 text-sm backdrop-blur-xl',
        statusStyles[integrity.status]
      )}
      initial={{ opacity: 0, y: -6 }}
      transition={{ duration: timing.normal, ease: premiumEase }}
    >
      <span className="relative flex h-5 w-5 items-center justify-center">
        <span className="absolute h-5 w-5 rounded-full bg-current opacity-15" />
        <Icon className="relative h-3.5 w-3.5" aria-hidden="true" />
      </span>
      <span className="font-medium">{integrity.label}</span>
      {fingerprint ? (
        <span className="hidden font-mono text-xs text-neutral-400 sm:inline">
          {shortFingerprint(fingerprint)}
        </span>
      ) : (
        <span className="hidden items-center gap-1.5 text-xs text-neutral-500 sm:inline-flex">
          <Radio className="h-3 w-3" aria-hidden="true" />
          {runtime === 'tauri' ? 'Listening' : 'Tauri required'}
        </span>
      )}
    </motion.div>
  );
}

function iconForStatus(status: IntegrityStatus) {
  if (status === 'verified') {
    return ShieldCheck;
  }

  if (status === 'degraded' || status === 'compromised') {
    return ShieldAlert;
  }

  return ShieldOff;
}

function shortFingerprint(fingerprint: string) {
  if (fingerprint.length <= 18) {
    return fingerprint;
  }

  return `${fingerprint.slice(0, 10)}...${fingerprint.slice(-6)}`;
}
