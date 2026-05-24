'use client';

import { motion } from 'framer-motion';
import { Activity, Database, Hash, RefreshCw, Shield, Zap } from 'lucide-react';
import { premiumEase, timing } from '@/animations/timing';
import { useEntropyStore } from '@/store/entropyStore';

const formatNum = (n: number) =>
  n >= 10000 ? `${(n / 1000).toFixed(1)}k` : n.toLocaleString();

export function PipelineStats() {
  const stats = useEntropyStore((s) => s.stats);
  const integrity = useEntropyStore((s) => s.integrity);

  const items = [
    {
      label: 'Pool Entropy',
      value: stats ? `${stats.pool_entropy_bits.toFixed(1)} bits` : '--',
      icon: Zap,
      accent: 'text-emerald-400'
    },
    {
      label: 'Health',
      value: stats?.health_status ?? integrity.status,
      icon: Shield,
      accent:
        (stats?.health_status ?? integrity.status) === 'healthy' || integrity.status === 'verified'
          ? 'text-emerald-400'
          : 'text-amber-400'
    },
    {
      label: 'Pool Fills',
      value: stats ? formatNum(stats.pool_fills) : '--',
      icon: Database,
      accent: 'text-cyan-400'
    },
    {
      label: 'Bytes Out',
      value: stats ? formatNum(stats.bytes_generated) : '--',
      icon: Hash,
      accent: 'text-cyan-400'
    },
    {
      label: 'DRBG Reseeds',
      value: stats ? formatNum(stats.reseed_count) : '--',
      icon: RefreshCw,
      accent: 'text-amber-400'
    },
    {
      label: 'Well Seeded',
      value: stats ? (stats.pool_well_seeded ? 'YES' : 'NO') : '--',
      icon: Activity,
      accent: stats?.pool_well_seeded ? 'text-emerald-400' : 'text-rose-400'
    }
  ];

  return (
    <div className="grid grid-cols-3 gap-2">
      {items.map((item, i) => {
        const Icon = item.icon;
        return (
          <motion.div
            key={item.label}
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: i * 0.04, duration: timing.normal, ease: premiumEase }}
            className="flex items-center gap-2.5 rounded-lg border border-white/[0.07] bg-white/[0.025] px-3 py-2.5"
          >
            <Icon className={`h-3.5 w-3.5 shrink-0 ${item.accent}`} />
            <div className="min-w-0">
              <p className="truncate text-[11px] leading-tight text-neutral-500">{item.label}</p>
              <p className="truncate font-mono text-sm font-medium text-neutral-100">{item.value}</p>
            </div>
          </motion.div>
        );
      })}
    </div>
  );
}
