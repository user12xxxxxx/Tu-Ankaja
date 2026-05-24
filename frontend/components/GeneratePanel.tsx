'use client';

import { useState } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { Fingerprint, KeyRound, Loader2, Minus, Plus, RectangleEllipsis } from 'lucide-react';
import { premiumEase, timing } from '@/animations/timing';
import { Button } from '@/components/ui/button';
import { generateEntropyMaterial, getEntropyIntegrity } from '@/services/tauriEntropy';
import { useEntropyStore } from '@/store/entropyStore';
import type { GeneratedMaterialKind } from '@/types/entropy';

const actions: Array<{
  kind: GeneratedMaterialKind;
  label: string;
  shortLabel: string;
  icon: typeof KeyRound;
}> = [
  { kind: 'aes-key', label: 'AES-256 Key', shortLabel: 'AES', icon: KeyRound },
  { kind: 'password', label: 'Password', shortLabel: 'Pass', icon: RectangleEllipsis },
  { kind: 'session-token', label: 'Session Token', shortLabel: 'Token', icon: Fingerprint }
];

export function GeneratePanel() {
  const [passwordLength, setPasswordLength] = useState(24);
  const entropy = useEntropyStore((state) => state.entropy);
  const runtime = useEntropyStore((state) => state.runtime);
  const generating = useEntropyStore((state) => state.generating);
  const lastError = useEntropyStore((state) => state.lastError);
  const addOutput = useEntropyStore((state) => state.addOutput);
  const setGenerating = useEntropyStore((state) => state.setGenerating);
  const setIntegrity = useEntropyStore((state) => state.setIntegrity);
  const setError = useEntropyStore((state) => state.setError);

  const handleGenerate = async (kind: GeneratedMaterialKind) => {
    setGenerating(kind, true);
    setError(undefined);

    try {
      const output = await generateEntropyMaterial(
        kind,
        { length: passwordLength },
        entropy.fingerprint
      );
      addOutput(output);

      getEntropyIntegrity()
        .then(setIntegrity)
        .catch(() => undefined);
    } catch (error) {
      setError(error instanceof Error ? error.message : 'Generation request failed');
    } finally {
      setGenerating(kind, false);
    }
  };

  const isUnavailable = false; // Works in both Tauri and HTTP mode now.

  return (
    <div>
      <div className="mb-2 flex items-center justify-between gap-3">
        <p className="text-[11px] uppercase tracking-wider text-neutral-500">Generate</p>
        <div className="flex items-center gap-1 rounded-md border border-white/[0.07] bg-white/[0.025] px-1">
          <Button
            aria-label="Decrease password length"
            disabled={passwordLength <= 12}
            onClick={() => setPasswordLength((v) => Math.max(12, v - 1))}
            size="icon"
            type="button"
            variant="ghost"
            className="h-6 w-6"
          >
            <Minus className="h-3 w-3" />
          </Button>
          <input
            aria-label="Password length"
            className="w-8 bg-transparent text-center font-mono text-xs text-neutral-100 outline-none"
            max={96}
            min={12}
            onChange={(e) => setPasswordLength(normalizeLength(e.target.value))}
            type="number"
            value={passwordLength}
          />
          <Button
            aria-label="Increase password length"
            disabled={passwordLength >= 96}
            onClick={() => setPasswordLength((v) => Math.min(96, v + 1))}
            size="icon"
            type="button"
            variant="ghost"
            className="h-6 w-6"
          >
            <Plus className="h-3 w-3" />
          </Button>
        </div>
      </div>

      <div className="flex gap-2">
        {actions.map((action) => {
          const Icon = action.icon;
          const isGenerating = generating[action.kind];

          return (
            <Button
              className="flex-1 gap-1.5 text-xs"
              disabled={isGenerating || isUnavailable}
              key={action.kind}
              onClick={() => handleGenerate(action.kind)}
              type="button"
              size="sm"
              variant={action.kind === 'aes-key' ? 'default' : 'secondary'}
            >
              {isGenerating ? (
                <Loader2 className="h-3.5 w-3.5 animate-spin" />
              ) : (
                <Icon className="h-3.5 w-3.5" />
              )}
              {action.label}
            </Button>
          );
        })}
      </div>

      <AnimatePresence>
        {lastError ? (
          <motion.p
            animate={{ opacity: 1, y: 0 }}
            className="mt-2 rounded-md border border-rose-300/20 bg-rose-400/10 px-2 py-1.5 text-[11px] text-rose-100"
            exit={{ opacity: 0, y: -4 }}
            initial={{ opacity: 0, y: 4 }}
            transition={{ duration: timing.fast, ease: premiumEase }}
          >
            {lastError}
          </motion.p>
        ) : null}
      </AnimatePresence>
    </div>
  );
}

function normalizeLength(value: string) {
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed)) return 24;
  return Math.min(96, Math.max(12, parsed));
}
