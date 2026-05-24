'use client';

import { useState } from 'react';
import { motion } from 'framer-motion';
import { Check, Copy, KeyRound, RectangleEllipsis, TicketCheck } from 'lucide-react';
import { premiumEase, timing } from '@/animations/timing';
import { Button } from '@/components/ui/button';
import type { GeneratedMaterialKind, GeneratedOutput } from '@/types/entropy';

const labels: Record<GeneratedMaterialKind, string> = {
  'aes-key': 'AES Key',
  password: 'Password',
  'session-token': 'Session Token'
};

export function OutputCard({ output, compact }: { output: GeneratedOutput; compact?: boolean }) {
  const [copied, setCopied] = useState(false);
  const Icon = iconForKind(output.kind);

  const handleCopy = async () => {
    await copyToClipboard(output.value);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1200);
  };

  if (compact) {
    return (
      <motion.div
        animate={{ opacity: 1, y: 0 }}
        className="rounded-lg border border-white/[0.07] bg-white/[0.025] px-3 py-2"
        initial={{ opacity: 0, y: 6 }}
        layout
        transition={{ duration: timing.fast, ease: premiumEase }}
      >
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2 min-w-0">
            <Icon className="h-3.5 w-3.5 shrink-0 text-emerald-300" />
            <span className="text-[11px] font-medium text-neutral-300">{labels[output.kind]}</span>
            <span className="text-[10px] text-neutral-600">{formatTimestamp(output.timestamp)}</span>
          </div>
          <Button
            aria-label="Copy"
            onClick={handleCopy}
            size="icon"
            type="button"
            variant="ghost"
            className="h-6 w-6"
          >
            {copied ? <Check className="h-3 w-3 text-emerald-300" /> : <Copy className="h-3 w-3" />}
          </Button>
        </div>
        <code className="mt-1 block truncate font-mono text-[11px] leading-relaxed text-neutral-100">
          {output.value}
        </code>
        {output.entropyFingerprint && (
          <p className="mt-0.5 truncate font-mono text-[10px] text-neutral-600">
            fp: {output.entropyFingerprint}
          </p>
        )}
      </motion.div>
    );
  }

  return (
    <motion.article
      animate={{ opacity: 1, y: 0 }}
      className="rounded-lg border border-white/10 bg-neutral-950/55 p-4 shadow-[0_20px_60px_rgba(0,0,0,0.25)]"
      initial={{ opacity: 0, y: 10 }}
      layout
      transition={{ duration: timing.normal, ease: premiumEase }}
    >
      <div className="mb-3 flex items-center justify-between gap-3">
        <div className="flex min-w-0 items-center gap-2">
          <span className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border border-white/10 bg-white/[0.055] text-emerald-200">
            <Icon className="h-4 w-4" aria-hidden="true" />
          </span>
          <div className="min-w-0">
            <h3 className="text-sm font-medium text-neutral-100">{labels[output.kind]}</h3>
            <p className="text-xs text-neutral-500">{formatTimestamp(output.timestamp)}</p>
          </div>
        </div>
        <Button aria-label="Copy generated value" onClick={handleCopy} size="icon" type="button" variant="ghost">
          {copied ? <Check className="h-4 w-4 text-emerald-200" /> : <Copy className="h-4 w-4" />}
        </Button>
      </div>

      <code className="block break-all rounded-lg border border-white/10 bg-black/30 p-3 font-mono text-sm leading-6 text-neutral-100">
        {output.value}
      </code>

      <div className="mt-3 flex items-center justify-between gap-3 text-xs text-neutral-500">
        <span>Entropy fingerprint</span>
        <span className="min-w-0 truncate font-mono text-neutral-300">
          {output.entropyFingerprint ?? 'Unavailable'}
        </span>
      </div>
    </motion.article>
  );
}

function iconForKind(kind: GeneratedMaterialKind) {
  if (kind === 'aes-key') return KeyRound;
  if (kind === 'password') return RectangleEllipsis;
  return TicketCheck;
}

function formatTimestamp(timestamp: string) {
  const date = new Date(timestamp);
  if (Number.isNaN(date.getTime())) return timestamp;
  return new Intl.DateTimeFormat(undefined, {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  }).format(date);
}

async function copyToClipboard(value: string) {
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(value);
    return;
  }

  const element = document.createElement('textarea');
  element.value = value;
  element.style.position = 'fixed';
  element.style.opacity = '0';
  document.body.appendChild(element);
  element.select();
  document.execCommand('copy');
  document.body.removeChild(element);
}
