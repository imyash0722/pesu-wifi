import React from 'react';
import { SpotlightCard } from './SpotlightCard';
import { ShieldCheck, Cpu, RefreshCw, Zap, Server, Smartphone, Lock, CheckCircle2 } from 'lucide-react';

export const BentoGrid: React.FC = () => {
  return (
    <section className="py-20 px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto w-full">
      <div className="text-center max-w-3xl mx-auto mb-16">
        <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 mb-4">
          <Zap className="w-3.5 h-3.5" />
          <span>Architected for 24/7 Resilience</span>
        </div>
        <h2 className="text-3xl sm:text-5xl font-extrabold tracking-tight text-white">
          Engineered to Outsmart <br className="hidden sm:inline" />
          <span className="bg-gradient-to-r from-emerald-400 via-teal-300 to-cyan-400 bg-clip-text text-transparent">
            Every Campus Drop Vector
          </span>
        </h2>
        <p className="mt-4 text-slate-400 text-base sm:text-lg">
          From Android 16 background limits to active VPN killswitches and router quota locks, PESU WiFi keeps you connected indefinitely.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {/* Card 1: Android 15/16 Standby Hardening (Col span 2 on md) */}
        <SpotlightCard className="md:col-span-2 group">
          <div className="flex flex-col h-full justify-between">
            <div>
              <div className="w-12 h-12 rounded-xl bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center text-emerald-400 mb-5">
                <Cpu className="w-6 h-6" />
              </div>
              <span className="text-xs font-mono font-semibold uppercase tracking-wider text-emerald-400">Android 14 · 15 · 16</span>
              <h3 className="text-2xl font-bold text-white mt-1 mb-3">
                Zero-Drop Standby &amp; Special Use Service
              </h3>
              <p className="text-slate-400 text-sm leading-relaxed mb-6">
                Android 15+ aggressively kills standard background services after 6 cumulative hours. PESU WiFi registers a dedicated <code className="text-emerald-300 bg-white/5 px-1.5 py-0.5 rounded">specialUse</code> foreground service with synchronized coroutine WakeLocks and an exact hardware RTC watchdog, ensuring zero drops in deep sleep.
              </p>
            </div>

            <div className="grid grid-cols-2 sm:grid-cols-3 gap-3 pt-4 border-t border-white/5 text-xs text-slate-300">
              <div className="flex items-center gap-2">
                <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
                <span>6hr FGS Kill Exemption</span>
              </div>
              <div className="flex items-center gap-2">
                <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
                <span>Doze CPU Synchronized</span>
              </div>
              <div className="flex items-center gap-2">
                <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
                <span>20s Handover WakeLock</span>
              </div>
            </div>
          </div>
        </SpotlightCard>

        {/* Card 2: Full VPN Compatibility */}
        <SpotlightCard className="group" spotlightColor="rgba(59, 130, 246, 0.2)">
          <div className="flex flex-col h-full justify-between">
            <div>
              <div className="w-12 h-12 rounded-xl bg-blue-500/10 border border-blue-500/20 flex items-center justify-center text-blue-400 mb-5">
                <ShieldCheck className="w-6 h-6" />
              </div>
              <span className="text-xs font-mono font-semibold uppercase tracking-wider text-blue-400">Resilient Sockets</span>
              <h3 className="text-2xl font-bold text-white mt-1 mb-3">
                Full VPN Compatibility
              </h3>
              <p className="text-slate-400 text-sm leading-relaxed mb-4">
                Tailscale, WireGuard, Cloudflare WARP, and AdGuard now run seamlessly. Our custom socket delegator catches Linux kernel <code className="text-blue-300 bg-white/5 px-1.5 py-0.5 rounded">EPERM</code> restrictions and falls back to local RFC-1918 gateway bypass.
              </p>
            </div>

            <div className="pt-4 border-t border-white/5 flex items-center gap-2 text-xs text-blue-300">
              <span className="h-2 w-2 rounded-full bg-blue-400 animate-ping" />
              <span>Tailscale · WireGuard · WARP · OpenVPN</span>
            </div>
          </div>
        </SpotlightCard>

        {/* Card 3: Multi-Account Auto-Failover */}
        <SpotlightCard className="group" spotlightColor="rgba(245, 158, 11, 0.18)">
          <div className="flex flex-col h-full justify-between">
            <div>
              <div className="w-12 h-12 rounded-xl bg-amber-500/10 border border-amber-500/20 flex items-center justify-center text-amber-400 mb-5">
                <RefreshCw className="w-6 h-6" />
              </div>
              <span className="text-xs font-mono font-semibold uppercase tracking-wider text-amber-400">Smart Failover</span>
              <h3 className="text-xl font-bold text-white mt-1 mb-3">
                Multi-Account Auto-Failover
              </h3>
              <p className="text-slate-400 text-sm leading-relaxed">
                When your active account hits quota or device limits, the engine silently authenticates with your next saved account without dropping your session.
              </p>
            </div>
            <div className="mt-4 pt-3 border-t border-white/5 text-xs text-amber-300/80">
              Zero manual re-logins during class exams
            </div>
          </div>
        </SpotlightCard>

        {/* Card 4: 120s ±30s Jittered Keepalive */}
        <SpotlightCard className="group">
          <div className="flex flex-col h-full justify-between">
            <div>
              <div className="w-12 h-12 rounded-xl bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center text-emerald-400 mb-5">
                <Zap className="w-6 h-6" />
              </div>
              <span className="text-xs font-mono font-semibold uppercase tracking-wider text-emerald-400">Dead Client Detection</span>
              <h3 className="text-xl font-bold text-white mt-1 mb-3">
                120s ±30s Randomized Jitter
              </h3>
              <p className="text-slate-400 text-sm leading-relaxed">
                Satisfies Cyberoam’s strict 180s DCD heartbeat requirement while preventing DDoS-like synchronized server floods across thousands of devices.
              </p>
            </div>
            <div className="mt-4 pt-3 border-t border-white/5 text-xs text-emerald-300/80">
              Clamped between 90s–150s dynamically
            </div>
          </div>
        </SpotlightCard>

        {/* Card 5: Sophos HTTPS & Dual-Gateway Probing */}
        <SpotlightCard className="group">
          <div className="flex flex-col h-full justify-between">
            <div>
              <div className="w-12 h-12 rounded-xl bg-teal-500/10 border border-teal-500/20 flex items-center justify-center text-teal-400 mb-5">
                <Server className="w-6 h-6" />
              </div>
              <span className="text-xs font-mono font-semibold uppercase tracking-wider text-teal-400">Dynamic Multi-Subnet</span>
              <h3 className="text-xl font-bold text-white mt-1 mb-3">
                PESU &amp; Amaatra Support
              </h3>
              <p className="text-slate-400 text-sm leading-relaxed">
                Seamlessly probes and authenticates with RR Campus &amp; EC Campus gateways (<code className="text-teal-300 text-xs">192.168.254.1</code>) and Amaatra Hostel (<code className="text-teal-300 text-xs">172.16.1.1</code>) with self-signed SSL trust.
              </p>
            </div>
            <div className="mt-4 pt-3 border-t border-white/5 text-xs text-teal-300/80">
              Captive 302/307 redirect auto-discovery
            </div>
          </div>
        </SpotlightCard>

        {/* Card 6: iOS Companion Solution (Col span 3) */}
        <SpotlightCard className="md:col-span-3 group" spotlightColor="rgba(168, 85, 247, 0.15)">
          <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-6">
            <div className="max-w-2xl">
              <div className="w-12 h-12 rounded-xl bg-purple-500/10 border border-purple-500/20 flex items-center justify-center text-purple-400 mb-4">
                <Smartphone className="w-6 h-6" />
              </div>
              <span className="text-xs font-mono font-semibold uppercase tracking-wider text-purple-400">Cross-Platform Companion</span>
              <h3 className="text-2xl font-bold text-white mt-1 mb-2">
                Have an iPhone or iPad? Zero-Click Apple Shortcuts
              </h3>
              <p className="text-slate-400 text-sm leading-relaxed">
                Use native Apple Shortcuts Personal Automations to trigger silent authentication the instant you step into range of campus Wi-Fi. No background app required on iOS.
              </p>
            </div>

            <div className="flex flex-col sm:flex-row gap-3 w-full md:w-auto shrink-0">
              <a
                href="./ios.html"
                className="px-5 py-3 rounded-xl bg-purple-500/15 hover:bg-purple-500/25 border border-purple-500/30 text-purple-300 text-sm font-semibold text-center transition-all"
              >
                View iOS Setup Guide &rarr;
              </a>
              <a
                href="./instructions.html"
                className="px-5 py-3 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-slate-300 text-sm font-semibold text-center transition-all"
              >
                Visual Android Guide
              </a>
            </div>
          </div>
        </SpotlightCard>
      </div>
    </section>
  );
};
