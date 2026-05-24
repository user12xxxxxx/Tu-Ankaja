'use client';

import { motion } from 'framer-motion';
import { premiumEase, timing } from '@/animations/timing';
import { useEntropyStore } from '@/store/entropyStore';

const tierColor: Record<string, string> = {
  excellent: 'text-emerald-400 border-emerald-400/30',
  adequate: 'text-cyan-400 border-cyan-400/30',
  degraded: 'text-amber-400 border-amber-400/30',
  failed: 'text-rose-400 border-rose-400/30',
  unknown: 'text-neutral-500 border-neutral-500/30'
};

export function SourceQuality() {
  const stats = useEntropyStore((s) => s.stats);
  const sources = stats?.source_quality ?? [];

  if (sources.length === 0) {
    return (
      <div className="rounded-lg border border-white/[0.07] bg-white/[0.025] px-3 py-3">
        <p className="text-[11px] uppercase tracking-wider text-neutral-500">Source Quality</p>
        <p className="mt-1 font-mono text-xs text-neutral-600">Awaiting data...</p>
      </div>
    );
  }

  return (
    <div className="space-y-2">
      <p className="text-[11px] uppercase tracking-wider text-neutral-500">Source Quality</p>
      {sources.map((source, i) => {
        const color = tierColor[source.tier] ?? tierColor.unknown;
        const entropyPct = Math.min(100, (source.min_entropy_bits_per_byte / 8) * 100);

        return (
          <motion.div
            key={source.source_id}
            initial={{ opacity: 0, x: 8 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: i * 0.06, duration: timing.normal, ease: premiumEase }}
            className="rounded-lg border border-white/[0.07] bg-white/[0.025] px-3 py-2.5"
          >
            <div className="flex items-center justify-between gap-2">
              <div className="flex items-center gap-2">
                <span className="font-mono text-xs text-neutral-400">
                  0x{source.source_id.toString(16).toUpperCase().padStart(2, '0')}
                </span>
                <span className={`rounded border px-1.5 py-0.5 text-[10px] font-semibold uppercase ${color}`}>
                  {source.tier}
                </span>
              </div>
              <span className="font-mono text-xs text-neutral-300">
                {source.observations} obs
              </span>
            </div>

            <div className="mt-2 flex items-center gap-3">
              <div className="flex-1">
                <div className="flex items-center justify-between text-[10px] text-neutral-500">
                  <span>H_min</span>
                  <span className="font-mono text-neutral-300">
                    {source.min_entropy_bits_per_byte.toFixed(2)} b/B
                  </span>
                </div>
                <div className="mt-1 h-1 overflow-hidden rounded-full bg-white/[0.06]">
                  <motion.div
                    className="h-full rounded-full bg-gradient-to-r from-emerald-500 to-cyan-400"
                    initial={{ width: 0 }}
                    animate={{ width: `${entropyPct}%` }}
                    transition={{ duration: 0.8, ease: premiumEase }}
                  />
                </div>
              </div>
              <div className="text-right">
                <p className="text-[10px] text-neutral-500">Conf</p>
                <p className="font-mono text-xs text-neutral-300">{(source.confidence * 100).toFixed(0)}%</p>
              </div>
            </div>

            <p className="mt-1.5 text-[10px] text-neutral-600">
              {source.total_bytes.toLocaleString()} bytes ingested
            </p>
          </motion.div>
        );
      })}
    </div>
  );
}
