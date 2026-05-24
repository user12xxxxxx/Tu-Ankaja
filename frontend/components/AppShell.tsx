'use client';

import { AnimatePresence, motion } from 'framer-motion';
import { Cpu, Radio } from 'lucide-react';
import { premiumEase, timing } from '@/animations/timing';
import { EntropyCanvas } from '@/components/EntropyCanvas';
import { GeneratePanel } from '@/components/GeneratePanel';
import { IntegrityIndicator } from '@/components/IntegrityIndicator';
import { LiveMetrics } from '@/components/LiveMetrics';
import { OutputCard } from '@/components/OutputCard';
import { PipelineStats } from '@/components/PipelineStats';
import { SecurityFeed } from '@/components/SecurityFeed';
import { SourceQuality } from '@/components/SourceQuality';
import { useEntropyEngine } from '@/hooks/useEntropyEngine';
import { useEntropyStore } from '@/store/entropyStore';

export function AppShell() {
  useEntropyEngine();

  const outputs = useEntropyStore((s) => s.outputs);
  const stats = useEntropyStore((s) => s.stats);
  const runtime = useEntropyStore((s) => s.runtime);

  return (
    <main className="relative h-screen overflow-hidden bg-[#050607] text-neutral-100">
      {/* Background gradients */}
      <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_50%_0%,rgba(69,239,147,0.07),transparent_32%),radial-gradient(circle_at_80%_18%,rgba(246,196,91,0.04),transparent_28%)]" />
      <div className="pointer-events-none absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-emerald-200/40 to-transparent" />

      <motion.div
        animate={{ opacity: 1 }}
        className="relative flex h-full flex-col p-3 sm:p-4"
        initial={{ opacity: 0 }}
        transition={{ duration: timing.slow, ease: premiumEase }}
      >
        {/* ─── HEADER ─── */}
        <header className="z-10 flex items-center justify-between gap-4 pb-3">
          <div className="flex items-center gap-3">
            <span className="flex h-9 w-9 items-center justify-center rounded-lg border border-white/10 bg-white/[0.055] text-emerald-200 shadow-[0_0_40px_rgba(69,239,147,0.12)]">
              <Cpu className="h-4 w-4" />
            </span>
            <div>
              <p className="text-sm font-medium text-neutral-100">Entropy Vault</p>
              <p className="text-[11px] text-neutral-500">
                {runtime === 'tauri' ? 'Hardware Entropy Engine' : 'MQTT Entropy Engine'}
              </p>
            </div>
          </div>

          <div className="flex items-center gap-3">
            {stats && (
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                className="hidden items-center gap-2 rounded-lg border border-white/[0.07] bg-white/[0.025] px-3 py-1.5 text-xs sm:flex"
              >
                <Radio className="h-3 w-3 text-emerald-400" />
                <span className="font-mono text-neutral-300">
                  {stats.pool_entropy_bits.toFixed(0)} bits
                </span>
                <span className="text-neutral-600">|</span>
                <span className="font-mono text-neutral-300">
                  {stats.bytes_generated.toLocaleString()} B
                </span>
              </motion.div>
            )}
            <IntegrityIndicator />
          </div>
        </header>

        {/* ─── MAIN GRID ─── */}
        <div className="z-10 grid min-h-0 flex-1 grid-cols-[1fr_1fr] gap-3 lg:grid-cols-[1.1fr_0.9fr]">
          {/* ─── LEFT COLUMN ─── */}
          <div className="flex min-h-0 flex-col gap-3">
            {/* Canvas hero */}
            <section className="relative min-h-0 flex-1 overflow-hidden rounded-lg border border-white/[0.07] bg-[#07090a] shadow-[0_40px_120px_rgba(0,0,0,0.42)]">
              <EntropyCanvas />
              <div className="pointer-events-none absolute inset-0 bg-[linear-gradient(180deg,rgba(5,6,7,0.1),rgba(5,6,7,0)_30%,rgba(5,6,7,0.7))]" />
              <LiveMetrics />
            </section>

            {/* Generate row */}
            <section className="rounded-lg border border-white/[0.07] bg-white/[0.03] p-3 backdrop-blur-xl">
              <GeneratePanel />
            </section>
          </div>

          {/* ─── RIGHT COLUMN ─── */}
          <div className="flex min-h-0 flex-col gap-3">
            {/* Pipeline stats grid */}
            <section>
              <p className="mb-1.5 text-[11px] uppercase tracking-wider text-neutral-500">Pipeline</p>
              <PipelineStats />
            </section>

            {/* Source quality */}
            <section className="min-h-0 shrink-0">
              <SourceQuality />
            </section>

            {/* Generated outputs */}
            <section className="flex min-h-0 flex-1 flex-col">
              <div className="mb-1.5 flex items-center justify-between">
                <p className="text-[11px] uppercase tracking-wider text-neutral-500">Output</p>
                <span className="text-[10px] text-neutral-600">{outputs.length} generated</span>
              </div>
              <div className="min-h-0 flex-1 space-y-2 overflow-y-auto scrollbar-hide">
                <AnimatePresence mode="popLayout">
                  {outputs.length > 0 ? (
                    outputs.map((output) => (
                      <OutputCard key={output.id} output={output} compact />
                    ))
                  ) : (
                    <motion.div
                      animate={{ opacity: 1 }}
                      className="flex h-full items-center justify-center rounded-lg border border-dashed border-white/[0.07] bg-white/[0.015] text-[11px] text-neutral-600"
                      initial={{ opacity: 0 }}
                    >
                      Generate material to see output
                    </motion.div>
                  )}
                </AnimatePresence>
              </div>
            </section>

            {/* Security events */}
            <section className="min-h-0 shrink-0">
              <SecurityFeed />
            </section>
          </div>
        </div>
      </motion.div>
    </main>
  );
}
