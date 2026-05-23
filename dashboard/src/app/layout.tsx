'use client';

import { useEffect, useState } from 'react';
import { Inter } from 'next/font/google';
import { ThemeProvider } from 'next-themes';
import Sidebar from '@/components/Sidebar';
import type { HealthStatus } from '@/lib/types';
import './globals.css';

const inter = Inter({ subsets: ['latin'], variable: '--font-inter' });

export default function RootLayout({ children }: { children: React.ReactNode }) {
  const [health, setHealth] = useState<HealthStatus | null>(null);

  useEffect(() => {
    const check = async () => {
      try {
        const res = await fetch(`${process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000'}/api/health`);
        if (res.ok) setHealth(await res.json());
      } catch {
        setHealth(null);
      }
    };
    check();
    const interval = setInterval(check, 30000);
    return () => clearInterval(interval);
  }, []);

  const statusColor = health?.status === 'healthy' ? 'bg-emerald-500' : health?.status === 'degraded' ? 'bg-amber-500' : 'bg-red-500';

  return (
    <html lang="en" suppressHydrationWarning>
      <body className={`${inter.variable} font-sans`}>
        <ThemeProvider attribute="class" defaultTheme="dark" enableSystem={false}>
          <div className="flex min-h-screen">
            <Sidebar />
            <main className="flex-1 min-w-0">
              <header className="sticky top-0 z-20 bg-white/80 dark:bg-surface-900/80 backdrop-blur-md border-b border-surface-200 dark:border-surface-800">
                <div className="flex items-center justify-between px-6 py-3">
                  <div className="lg:hidden" />
                  <div className="flex items-center gap-4 ml-auto">
                    <div className="flex items-center gap-2">
                      <span className={`w-2 h-2 rounded-full ${statusColor} animate-pulse`} />
                      <span className="text-xs text-surface-500 dark:text-surface-400 capitalize">
                        {health?.status || 'disconnected'}
                      </span>
                    </div>
                    {health && (
                      <span className="text-xs text-surface-400 dark:text-surface-500">
                        v{health.version}
                      </span>
                    )}
                  </div>
                </div>
              </header>
              <div className="p-6">
                {children}
              </div>
            </main>
          </div>
        </ThemeProvider>
      </body>
    </html>
  );
}
