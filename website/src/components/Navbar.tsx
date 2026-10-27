import React from 'react';
import { Wifi, BookOpen, Smartphone, Download } from 'lucide-react';

const GithubIcon = ({ className = 'w-4 h-4' }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path fillRule="evenodd" clipRule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" />
  </svg>
);

export const Navbar: React.FC = () => {
  return (
    <header className="sticky top-0 z-50 w-full border-b border-white/5 bg-[#08090d]/80 backdrop-blur-xl">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between">
        {/* Brand */}
        <a href="./index.html" className="flex items-center gap-3 group">
          <div className="h-9 w-9 rounded-xl bg-gradient-to-tr from-emerald-500 to-teal-400 p-0.5 flex items-center justify-center shadow-lg shadow-emerald-500/20 group-hover:scale-105 transition-transform">
            <div className="h-full w-full bg-[#0a0c12] rounded-[10px] flex items-center justify-center">
              <Wifi className="w-5 h-5 text-emerald-400" />
            </div>
          </div>
          <div className="flex items-center gap-2">
            <span className="font-bold text-lg text-white tracking-tight">PESU WiFi</span>
            <span className="text-[11px] font-mono px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
              v3.7.5
            </span>
          </div>
        </a>

        {/* Navigation Links */}
        <nav className="hidden md:flex items-center gap-6 text-sm font-medium text-slate-300">
          <a
            href="./instructions.html"
            className="flex items-center gap-1.5 hover:text-emerald-400 transition-colors"
          >
            <BookOpen className="w-4 h-4" />
            <span>Setup Guide</span>
          </a>
          <a
            href="./ios.html"
            className="flex items-center gap-1.5 hover:text-purple-400 transition-colors"
          >
            <Smartphone className="w-4 h-4" />
            <span>iOS Guide</span>
          </a>
          <a
            href="https://github.com/imyash0722/pesu-wifi"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-1.5 hover:text-white transition-colors"
          >
            <GithubIcon className="w-4 h-4" />
            <span>GitHub</span>
          </a>
        </nav>

        {/* CTA Button */}
        <div className="flex items-center gap-3">
          <a
            href="#download"
            className="flex items-center gap-2 px-4 py-2 rounded-xl text-xs sm:text-sm font-semibold bg-emerald-500 hover:bg-emerald-400 text-black shadow-md shadow-emerald-500/20 hover:shadow-emerald-500/40 transition-all active:scale-95"
          >
            <Download className="w-4 h-4" />
            <span>Download v3.7.5</span>
          </a>
        </div>
      </div>
    </header>
  );
};
