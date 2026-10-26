import React from 'react';
import { AuroraBackground } from './components/AuroraBackground';
import { Navbar } from './components/Navbar';
import { BentoGrid } from './components/BentoGrid';
import { InteractiveSimulator } from './components/InteractiveSimulator';
import { DownloadSection } from './components/DownloadSection';
import { Footer } from './components/Footer';
import { ShinyButton } from './components/ShinyButton';
import { Download, BookOpen, Smartphone, Shield, Radio, CheckCircle, ExternalLink, ArrowRight } from 'lucide-react';

export const App: React.FC = () => {
  return (
    <AuroraBackground>
      <Navbar />

      <main className="flex-1 flex flex-col items-center">
        {/* HERO SECTION */}
        <section className="relative pt-20 pb-16 px-4 sm:px-6 lg:px-8 max-w-6xl mx-auto text-center flex flex-col items-center">
          {/* Notification / Version Announcement Chip */}
          <div className="inline-flex items-center gap-2 px-3.5 py-1.5 rounded-full text-xs font-medium bg-emerald-500/10 border border-emerald-500/30 text-emerald-400 mb-8 backdrop-blur-md shadow-lg shadow-emerald-500/5 hover:border-emerald-500/50 transition-colors">
            <span className="flex h-2 w-2 rounded-full bg-emerald-400 animate-ping" />
            <span>v3.7.0 Released: Unified Multi-Platform (Android · Windows · Linux)</span>
            <ArrowRight className="w-3.5 h-3.5" />
          </div>

          {/* Hero Headline */}
          <h1 className="text-4xl sm:text-6xl lg:text-7xl font-black tracking-tight text-white max-w-4xl leading-[1.1]">
            Campus Wi-Fi, <br className="hidden sm:inline" />
            <span className="bg-gradient-to-r from-emerald-400 via-teal-300 to-cyan-400 bg-clip-text text-transparent">
              Always Connected.
            </span>
          </h1>

          {/* Subtitle */}
          <p className="mt-6 text-base sm:text-xl text-slate-300 max-w-2xl mx-auto leading-relaxed font-normal">
            Automated 1-tap sign-in and continuous 120s jittered keepalive for PES University Cyberroam gateways. Built with native Android foreground architecture to survive deep sleep, AP roaming, and active VPN tunnels.
          </p>

          {/* Call to Actions */}
          <div className="mt-10 flex flex-wrap items-center justify-center gap-4">
            <a href="#download">
              <ShinyButton variant="primary" icon={<Download className="w-5 h-5" />} className="text-base px-7 py-3.5">
                Download Latest APK (v1.7.5)
              </ShinyButton>
            </a>

            <a href="./instructions.html">
              <ShinyButton variant="secondary" icon={<BookOpen className="w-5 h-5" />} className="text-base px-6 py-3.5">
                Visual Setup Guide
              </ShinyButton>
            </a>

            <a href="./ios.html">
              <ShinyButton variant="outline" icon={<Smartphone className="w-5 h-5" />} className="text-base px-6 py-3.5">
                iOS Guide
              </ShinyButton>
            </a>
          </div>

          {/* Trust Badges */}
          <div className="mt-12 flex flex-wrap items-center justify-center gap-y-3 gap-x-8 text-xs text-slate-400">
            <div className="flex items-center gap-2">
              <CheckCircle className="w-4 h-4 text-emerald-400" />
              <span>Android 8 through 16 Compatible</span>
            </div>
            <div className="flex items-center gap-2">
              <Shield className="w-4 h-4 text-emerald-400" />
              <span>Tailscale &amp; WireGuard Resilient</span>
            </div>
            <div className="flex items-center gap-2">
              <Radio className="w-4 h-4 text-emerald-400" />
              <span>EC Campus, RR Campus &amp; Amaatra</span>
            </div>
          </div>
        </section>

        {/* INTERACTIVE SIMULATOR SHOWCASE */}
        <section className="py-12 px-4 sm:px-6 lg:px-8 max-w-5xl mx-auto w-full">
          <InteractiveSimulator />
        </section>

        {/* BENTO GRID (FEATURES) */}
        <BentoGrid />

        {/* DOWNLOAD SECTION */}
        <DownloadSection />
      </main>

      <Footer />
    </AuroraBackground>
  );
};

export default App;
