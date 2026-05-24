import './globals.css';
import type { Metadata } from 'next';
import { NavBar } from '@/components/NavBar';

export const metadata: Metadata = {
  title: 'TU Ankaja — OTP Generator',
  description: 'MQTT-powered OTP generation engine'
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body className="bg-[#050607] text-neutral-100">
        <div className="flex h-screen flex-col overflow-hidden">
          <NavBar />
          <main className="flex-1 overflow-y-auto">{children}</main>
        </div>
      </body>
    </html>
  );
}
