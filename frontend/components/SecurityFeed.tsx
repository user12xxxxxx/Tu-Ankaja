'use client';

import { AnimatePresence, motion } from 'framer-motion';
import { ShieldAlert } from 'lucide-react';
import { premiumEase, timing } from '@/animations/timing';
import { useEntropyStore } from '@/store/entropyStore';

export function SecurityFeed() {
  const events = useEntropyStore((s) => s.securityEvents);

  return (
    <div className="flex min-h-0 flex-col">
      <p className="mb-1.5 text-[11px] uppercase tracking-wider text-neutral-500">Security Events</p>
      <div className="min-h-0 flex-1 overflow-y-auto scrollbar-hide">
        <AnimatePresence initial={false}>
          {events.length === 0 ? (
            <motion.p
              key="empty"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              className="py-2 font-mono text-[11px] text-neutral-600"
            >
              No events recorded
            </motion.p>
          ) : (
            events.slice(0, 8).map((event, i) => (
              <motion.div
                key={`${event.kind}-${event.timestamp}-${i}`}
                initial={{ opacity: 0, x: -6 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: 6 }}
                transition={{ duration: timing.fast, ease: premiumEase }}
                className="flex items-start gap-2 border-b border-white/[0.04] py-1.5"
              >
                <ShieldAlert className="mt-0.5 h-3 w-3 shrink-0 text-amber-400/70" />
                <div className="min-w-0 flex-1">
                  <p className="truncate font-mono text-[11px] text-neutral-300">{event.detail || event.kind}</p>
                  <p className="text-[10px] text-neutral-600">{formatEventTime(event.timestamp)}</p>
                </div>
              </motion.div>
            ))
          )}
        </AnimatePresence>
      </div>
    </div>
  );
}

function formatEventTime(timestamp: string) {
  const date = new Date(timestamp);
  if (Number.isNaN(date.getTime())) return timestamp;
  return date.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit', second: '2-digit' });
}
