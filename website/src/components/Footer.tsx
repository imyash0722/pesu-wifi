import React from 'react';
import { Wifi, ExternalLink, Heart } from 'lucide-react';

const GithubIcon = ({ className = 'w-3.5 h-3.5' }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path fillRule="evenodd" clipRule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" />
  </svg>
);

export const Footer: React.FC = () => {
  return (
    <footer className="border-t border-white/5 bg-[#07080b] py-12 px-4 sm:px-6 lg:px-8 mt-auto">
      <div className="max-w-7xl mx-auto flex flex-col md:flex-row items-center justify-between gap-6">
        <div className="flex items-center gap-3">
          <div className="h-7 w-7 rounded-lg bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center text-emerald-400">
            <Wifi className="w-4 h-4" />
          </div>
          <span className="font-semibold text-sm text-slate-300">PESU WiFi Project</span>
          <span className="text-slate-600">•</span>
          <span className="text-xs text-slate-500">MIT Open Source</span>
        </div>

        <div className="flex flex-wrap items-center justify-center gap-6 text-xs text-slate-400">
          <a href="./instructions.html" className="hover:text-emerald-400 transition-colors">
            Visual Android Guide
          </a>
          <a href="./ios.html" className="hover:text-purple-400 transition-colors">
            iOS Automation Guide
          </a>
          <a
            href="https://github.com/imyash0722/pesu-wifi/releases"
            target="_blank"
            rel="noopener noreferrer"
            className="hover:text-white transition-colors"
          >
            Release History
          </a>
          <a
            href="https://github.com/imyash0722/pesu-wifi/blob/master/CHANGELOG.md"
            target="_blank"
            rel="noopener noreferrer"
            className="hover:text-white transition-colors"
          >
            Changelog
          </a>
          <a
            href="https://github.com/imyash0722/pesu-wifi"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-1 hover:text-white transition-colors"
          >
            <GithubIcon className="w-3.5 h-3.5" />
            <span>Repository</span>
          </a>
        </div>

        <div className="text-xs text-slate-500 text-center md:text-right">
          <span>Developed with care for PES University students &amp; faculty.</span>
        </div>
      </div>
    </footer>
  );
};
