'use client';

import { useEffect } from 'react';
import {
  getEntropyIntegrity,
  getEntropyStats,
  getSecurityEvents,
  isTauriRuntime,
  listenForEntropyUpdates
} from '@/services/tauriEntropy';
import { useEntropyStore } from '@/store/entropyStore';

export function useEntropyEngine() {
  useEffect(() => {
    let mounted = true;
    let unlisten: (() => void) | undefined;
    let pollTimer: ReturnType<typeof setInterval> | undefined;

    const store = useEntropyStore.getState();
    store.setRuntime(isTauriRuntime() ? 'tauri' : 'browser');

    getEntropyIntegrity()
      .then((integrity) => {
        if (mounted) {
          useEntropyStore.getState().setIntegrity(integrity);
        }
      })
      .catch((error: unknown) => {
        if (mounted) {
          useEntropyStore.getState().setIntegrity({
            status: 'offline',
            label: errorToMessage(error),
            checkedAt: new Date().toISOString()
          });
        }
      });

    listenForEntropyUpdates((sample) => {
      useEntropyStore.getState().pushEntropySample(sample);
    })
      .then((cleanup) => {
        unlisten = cleanup;
      })
      .catch((error: unknown) => {
        if (mounted) {
          useEntropyStore.getState().setError(errorToMessage(error));
        }
      });

    const pollData = async () => {
      if (!mounted) return;

      const [stats, events] = await Promise.all([
        getEntropyStats().catch(() => null),
        getSecurityEvents().catch(() => [])
      ]);

      if (!mounted) return;

      if (stats) {
        useEntropyStore.getState().setStats(stats);
      }
      if (events.length > 0) {
        useEntropyStore.getState().pushSecurityEvents(events);
      }
    };

    pollData();
    pollTimer = setInterval(pollData, 1500);

    return () => {
      mounted = false;
      unlisten?.();
      if (pollTimer) clearInterval(pollTimer);
    };
  }, []);
}

function errorToMessage(error: unknown) {
  return error instanceof Error ? error.message : 'Entropy engine unavailable';
}
