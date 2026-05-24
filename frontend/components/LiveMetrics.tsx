'use client';

import { motion } from 'framer-motion';
import { premiumEase, timing } from '@/animations/timing';
import { useEntropyStore } from '@/store/entropyStore';

export function LiveMetrics() {
  const entropy = useEntropyStore((s) => s.entropy);
  const hasSignal = useEntropyStore((s) => s.hasEntropySignal);
  const runtime = useEntropyStore((s) => s.runtime);

  const intensityPct = Math.round(entropy.intensity * 100);
  const stabilityPct = Math.round(entropy.stability * 100);

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ delay: 0.2, duration: timing.slow, ease: premiumEase }}
      className="pointer-events-none absolute bottom-0 left-0 right-0 flex items-end justify-between gap-4 p-4"
    >
      <div>
        <p className="text-lg font-medium leading-tight text-neutral-50 drop-shadow-lg sm:text-2xl">
          Hardware entropy, condensed.
        </p>
        <p className="mt-1 text-xs text-neutral-400">
          {runtime === 'tauri' ? 'Connected via Tauri IPC' : 'Connected via HTTP API'}
        </p>
      </div>

      {hasSignal && (
        <div className="flex gap-4 rounded-lg border border-white/10 bg-black/60 px-3 py-2 backdrop-blur-xl">
          <Metric label="Intensity" value={`${intensityPct}%`} pct={entropy.intensity} color="emerald" />
          <Metric label="Stability" value={`${stabilityPct}%`} pct={entropy.stability} color="cyan" />
          {entropy.sequence !== undefined && (
            <div className="text-center">
              <p className="text-[10px] text-neutral-500">Seq</p>
              <p className="font-mono text-sm font-medium text-neutral-200">#{entropy.sequence}</p>
            </div>
          )}
        </div>
      )}
    </motion.div>
  );
}

function Metric({
  label,
  value,
  pct,
  color
}: {
  label: string;
  value: string;
  pct: number;
  color: 'emerald' | 'cyan';
}) {
  const barColor = color === 'emerald' ? 'bg-emerald-400' : 'bg-cyan-400';

  return (
    <div className="min-w-[60px] text-center">
      <p className="text-[10px] text-neutral-500">{label}</p>
      <p className="font-mono text-sm font-medium text-neutral-200">{value}</p>
      <div className="mt-1 h-0.5 w-full overflow-hidden rounded-full bg-white/10">
        <motion.div
          className={`h-full rounded-full ${barColor}`}
          animate={{ width: `${pct * 100}%` }}
          transition={{ duration: 0.5 }}
        />
      </div>
    </div>
  );
}
