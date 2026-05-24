'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { motion } from 'framer-motion';
import { Cpu, LogIn, Loader2 } from 'lucide-react';

export default function LoginPage() {
  const router = useRouter();
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const handleLogin = (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    if (!username.trim() || !password.trim()) {
      setError('Please enter username and password');
      return;
    }

    setLoading(true);
    // Simple client-side auth (no backend auth for now)
    setTimeout(() => {
      sessionStorage.setItem('ev_logged_in', '1');
      sessionStorage.setItem('ev_user', username);
      router.push('/otp');
    }, 600);
  };

  return (
    <div className="flex min-h-screen items-center justify-center px-4">
      {/* Background gradients */}
      <div className="pointer-events-none fixed inset-0 bg-[radial-gradient(circle_at_50%_40%,rgba(69,239,147,0.06),transparent_50%)]" />

      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        className="relative w-full max-w-sm"
      >
        {/* Logo */}
        <div className="mb-8 flex flex-col items-center gap-3">
          <span className="flex h-14 w-14 items-center justify-center rounded-2xl border border-white/10 bg-white/[0.055] text-emerald-300 shadow-[0_0_60px_rgba(69,239,147,0.15)]">
            <Cpu className="h-7 w-7" />
          </span>
          <div className="text-center">
            <h1 className="text-xl font-semibold text-neutral-100">TU Ankaja</h1>
            <p className="mt-0.5 text-xs text-neutral-500">MQTT OTP Engine</p>
            <p className="mt-2 text-[11px] font-medium tracking-wide text-emerald-400/70">Team TU Ankaja</p>
            <p className="mt-0.5 text-[10px] text-neutral-600">IEEE MYSOSA</p>
          </div>
        </div>

        {/* Login Card */}
        <form
          onSubmit={handleLogin}
          className="rounded-xl border border-white/[0.07] bg-white/[0.03] p-6 backdrop-blur-xl"
        >
          <h2 className="mb-5 text-center text-sm font-medium text-neutral-300">
            Sign in to continue
          </h2>

          <div className="space-y-3">
            <div>
              <label htmlFor="username" className="mb-1 block text-[11px] uppercase tracking-wider text-neutral-500">
                Username
              </label>
              <input
                id="username"
                type="text"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                className="w-full rounded-lg border border-white/[0.07] bg-white/[0.03] px-3 py-2.5 text-sm text-neutral-100 outline-none placeholder:text-neutral-600 focus:border-emerald-500/40 focus:ring-1 focus:ring-emerald-500/20"
                placeholder="Enter username"
                autoComplete="username"
              />
            </div>

            <div>
              <label htmlFor="password" className="mb-1 block text-[11px] uppercase tracking-wider text-neutral-500">
                Password
              </label>
              <input
                id="password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="w-full rounded-lg border border-white/[0.07] bg-white/[0.03] px-3 py-2.5 text-sm text-neutral-100 outline-none placeholder:text-neutral-600 focus:border-emerald-500/40 focus:ring-1 focus:ring-emerald-500/20"
                placeholder="Enter password"
                autoComplete="current-password"
              />
            </div>
          </div>

          {error && (
            <motion.p
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              className="mt-3 rounded-md border border-rose-300/20 bg-rose-400/10 px-2 py-1.5 text-center text-xs text-rose-200"
            >
              {error}
            </motion.p>
          )}

          <button
            type="submit"
            disabled={loading}
            className="mt-5 flex w-full items-center justify-center gap-2 rounded-lg bg-emerald-500 py-2.5 text-sm font-medium text-black transition-colors hover:bg-emerald-400 disabled:opacity-50"
          >
            {loading ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <LogIn className="h-4 w-4" />
            )}
            Sign In
          </button>
        </form>

        <p className="mt-4 text-center text-[10px] text-neutral-600">
          Hardware entropy powered OTP generation — Team TU Ankaja | IEEE MYSOSA
        </p>
      </motion.div>
    </div>
  );
}
