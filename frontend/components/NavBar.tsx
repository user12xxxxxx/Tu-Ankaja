'use client';

import Link from 'next/link';
import { usePathname, useRouter } from 'next/navigation';
import { Cpu, KeyRound, Database, Shield, LogOut } from 'lucide-react';

const links = [
  { href: '/otp', label: 'OTP Generator', icon: KeyRound },
  { href: '/data', label: 'Raw Data', icon: Database },
  { href: '/entropy', label: 'Entropy Engine', icon: Shield },
];

export function NavBar() {
  const pathname = usePathname();
  const router = useRouter();

  // Hide navbar on login page
  if (pathname === '/') return null;

  const handleLogout = () => {
    sessionStorage.removeItem('ev_logged_in');
    sessionStorage.removeItem('ev_user');
    router.push('/');
  };

  return (
    <header className="z-10 flex items-center justify-between gap-4 border-b border-white/[0.07] px-4 py-3">
      <div className="flex items-center gap-3">
        <span className="flex h-9 w-9 items-center justify-center rounded-lg border border-white/10 bg-white/[0.055] text-emerald-200 shadow-[0_0_40px_rgba(69,239,147,0.12)]">
          <Cpu className="h-4 w-4" />
        </span>
        <div>
          <p className="text-sm font-medium text-neutral-100">TU Ankaja</p>
          <p className="text-[11px] text-neutral-500">MQTT OTP Engine</p>
        </div>
      </div>

      <div className="flex items-center gap-1">
        <nav className="flex items-center gap-1">
          {links.map((link) => {
            const Icon = link.icon;
            const isActive = pathname === link.href || pathname?.startsWith(link.href + '/');

            return (
              <Link
                key={link.href}
                href={link.href}
                className={`flex items-center gap-1.5 rounded-md px-3 py-1.5 text-xs transition-colors ${
                  isActive
                    ? 'bg-emerald-500/15 text-emerald-300'
                    : 'text-neutral-400 hover:bg-white/[0.05] hover:text-neutral-200'
                }`}
              >
                <Icon className="h-3.5 w-3.5" />
                {link.label}
              </Link>
            );
          })}
        </nav>

        <button
          onClick={handleLogout}
          className="ml-2 flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-xs text-neutral-500 transition-colors hover:bg-white/[0.05] hover:text-neutral-300"
        >
          <LogOut className="h-3.5 w-3.5" />
          Logout
        </button>
      </div>
    </header>
  );
}
