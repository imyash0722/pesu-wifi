import React from 'react';
import { Download, ShieldCheck, Terminal, Smartphone, Sparkles, Check, ArrowRight, Laptop, Monitor } from 'lucide-react';
import { SpotlightCard } from './SpotlightCard';
import { ShinyButton } from './ShinyButton';
import { CommandBlock } from './CommandBlock';

export const DownloadSection: React.FC = () => {
  const version = 'v3.7.6';
  const baseUrl = `https://github.com/imyash0722/pesu-wifi/releases/download/${version}`;

  const androidStableUrl = `${baseUrl}/pesu-wifi-${version}-android-stable.apk`;
  const androidTesterUrl = `${baseUrl}/pesu-wifi-${version}-android-tester.apk`;
  const windowsSetupUrl = `${baseUrl}/pesu-wifi-${version}-windows-x86_64-setup.exe`;
  const linuxTarballUrl = `${baseUrl}/pesu-wifi-${version}-linux-x86_64.tar.gz`;

  return (
    <section id="download" className="py-24 px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto w-full">
      <div className="text-center max-w-3xl mx-auto mb-16">
        <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 mb-4">
          <Download className="w-3.5 h-3.5" />
          <span>Unified Releases · All Platforms</span>
        </div>
        <h2 className="text-3xl sm:text-5xl font-extrabold tracking-tight text-white">
          Download PESU WiFi
        </h2>
        <p className="mt-4 text-slate-400 text-base sm:text-lg">
          Zero-drop background connectivity across Android, Windows, Linux, macOS, and iOS. Fully open-source and verified.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {/* Android Card */}
        <SpotlightCard className="relative group border-emerald-500/30 bg-[#0f1418]/90">
          <div className="absolute top-4 right-4 px-2.5 py-0.5 rounded-full bg-emerald-500/20 text-emerald-400 text-[11px] font-semibold border border-emerald-500/30 flex items-center gap-1">
            <Sparkles className="w-3 h-3" />
            <span>Mobile App</span>
          </div>

          <div className="flex flex-col h-full justify-between pt-2">
            <div>
              <div className="w-12 h-12 rounded-xl bg-emerald-500/20 border border-emerald-500/30 flex items-center justify-center text-emerald-400 mb-5">
                <Smartphone className="w-6 h-6" />
              </div>
              <h3 className="text-2xl font-bold text-white mb-1">Android App</h3>
              <p className="text-xs font-mono text-emerald-400 mb-4">{version} · Native Material 3</p>
              <p className="text-slate-400 text-sm leading-relaxed mb-6">
                Special Use foreground service, 120s jittered keepalive, multi-account auto-failover, and kernel EPERM resilient VPN bypass.
              </p>

              <ul className="space-y-2.5 text-xs text-slate-300 mb-8">
                <li className="flex items-center gap-2">
                  <Check className="w-4 h-4 text-emerald-400 shrink-0" />
                  <span>Android 8 through 16 compatibility</span>
                </li>
                <li className="flex items-center gap-2">
                  <Check className="w-4 h-4 text-emerald-400 shrink-0" />
                  <span>Vivo, Xiaomi, Samsung battery whitelist support</span>
                </li>
                <li className="flex items-center gap-2">
                  <Check className="w-4 h-4 text-emerald-400 shrink-0" />
                  <span>Quick Settings pull-down tile</span>
                </li>
              </ul>
            </div>

            <div className="space-y-2.5">
              <a href={androidStableUrl} className="block w-full">
                <ShinyButton variant="primary" className="w-full" icon={<Download className="w-4 h-4" />}>
                  Download Stable APK
                </ShinyButton>
              </a>
              <a href={androidTesterUrl} className="block w-full">
                <ShinyButton variant="secondary" className="w-full" icon={<Terminal className="w-4 h-4 text-slate-400" />}>
                  Download Tester APK (Logs)
                </ShinyButton>
              </a>
              <p className="text-[11px] text-slate-400 text-center pt-1 leading-normal">
                Upgrading from ≤v3.7.1? Please uninstall the old version once before installing v3.7.3 due to the new permanent signing key.
              </p>
            </div>
          </div>
        </SpotlightCard>

        {/* Windows Card */}
        <SpotlightCard className="group border-blue-500/20 bg-[#0d121c]/90" spotlightColor="rgba(59, 130, 246, 0.2)">
          <div className="flex flex-col h-full justify-between pt-2">
            <div>
              <div className="w-12 h-12 rounded-xl bg-blue-500/10 border border-blue-500/20 flex items-center justify-center text-blue-400 mb-5">
                <Monitor className="w-6 h-6" />
              </div>
              <h3 className="text-2xl font-bold text-white mb-1">Windows (PC)</h3>
              <p className="text-xs font-mono text-blue-400 mb-4">{version} · Win32 / WlanAPI</p>
              <p className="text-slate-400 text-sm leading-relaxed mb-6">
                Native Windows 10/11 service with Inno Setup installer, WinRT toast notifications, and automatic /32 host route VPN bypass.
              </p>

              <ul className="space-y-2.5 text-xs text-slate-300 mb-8">
                <li className="flex items-center gap-2">
                  <Check className="w-4 h-4 text-blue-400 shrink-0" />
                  <span>1-Click .exe Setup Wizard</span>
                </li>
                <li className="flex items-center gap-2">
                  <Check className="w-4 h-4 text-blue-400 shrink-0" />
                  <span>Scoop &amp; PowerShell install support</span>
                </li>
                <li className="flex items-center gap-2">
                  <Check className="w-4 h-4 text-blue-400 shrink-0" />
                  <span>Continuous background service</span>
                </li>
              </ul>
            </div>

            <div className="space-y-2.5">
              <a href={windowsSetupUrl} className="block w-full">
                <ShinyButton variant="primary" className="w-full bg-gradient-to-r from-blue-600 to-cyan-600 hover:from-blue-500 hover:to-cyan-500" icon={<Download className="w-4 h-4" />}>
                  Download Windows .exe Setup
                </ShinyButton>
              </a>
              <CommandBlock
                command="irm https://raw.githubusercontent.com/imyash0722/pesu-wifi/master/install.ps1 | iex"
                language="powershell"
                prompt="PS>"
              >
                <span className="text-cyan-400 font-semibold">irm</span>{' '}
                <span className="text-sky-300">https://raw.githubusercontent.com/imyash0722/pesu-wifi/master/install.ps1</span>{' '}
                <span className="text-purple-400 font-bold">|</span>{' '}
                <span className="text-cyan-400 font-semibold">iex</span>
              </CommandBlock>
            </div>
          </div>
        </SpotlightCard>

        {/* Linux & iOS Card */}
        <SpotlightCard className="group border-purple-500/20 bg-[#120f18]/80" spotlightColor="rgba(168, 85, 247, 0.15)">
          <div className="flex flex-col h-full justify-between pt-2">
            <div>
              <div className="w-12 h-12 rounded-xl bg-purple-500/10 border border-purple-500/20 flex items-center justify-center text-purple-400 mb-5">
                <Laptop className="w-6 h-6" />
              </div>
              <h3 className="text-2xl font-bold text-white mb-1">Linux &amp; iOS</h3>
              <p className="text-xs font-mono text-purple-400 mb-4">systemd &amp; Apple Shortcuts</p>
              <p className="text-slate-400 text-sm leading-relaxed mb-6">
                Full-featured systemd background daemon on Linux (Arch/Ubuntu/Fedora) and zero-click Apple Shortcuts personal automation for iPhone &amp; iPad.
              </p>

              <ul className="space-y-2.5 text-xs text-slate-300 mb-8">
                <li className="flex items-center gap-2">
                  <Check className="w-4 h-4 text-purple-400 shrink-0" />
                  <span>Arch Linux AUR: <code className="text-purple-300">pesu-wifi-bin</code></span>
                </li>
                <li className="flex items-center gap-2">
                  <Check className="w-4 h-4 text-purple-400 shrink-0" />
                  <span>Zero-click Apple Shortcuts automation</span>
                </li>
                <li className="flex items-center gap-2">
                  <Check className="w-4 h-4 text-purple-400 shrink-0" />
                  <span>Shared <code className="text-xs">config.json</code> format</span>
                </li>
              </ul>
            </div>

            <div className="space-y-2.5">
              <a href="./ios.html" className="block w-full">
                <ShinyButton
                  variant="outline"
                  className="w-full"
                  icon={<Smartphone className="w-4 h-4 text-purple-400" />}
                  rightIcon={<ArrowRight className="w-4 h-4 text-purple-400 group-hover:translate-x-1 transition-transform" />}
                >
                  Open iOS Shortcuts Guide
                </ShinyButton>
              </a>
              <a href={linuxTarballUrl} className="block w-full">
                <ShinyButton
                  variant="secondary"
                  className="w-full"
                  icon={<Terminal className="w-4 h-4 text-slate-400" />}
                  rightIcon={<ArrowRight className="w-4 h-4 text-slate-400 group-hover:translate-x-1 transition-transform" />}
                >
                  Download Linux tar.gz
                </ShinyButton>
              </a>
              <CommandBlock
                command="curl -sSL https://raw.githubusercontent.com/imyash0722/pesu-wifi/master/install.sh | bash"
                language="bash"
                prompt="$"
              >
                <span className="text-emerald-400 font-semibold">curl</span>{' '}
                <span className="text-amber-400 font-medium">-sSL</span>{' '}
                <span className="text-sky-300">https://raw.githubusercontent.com/imyash0722/pesu-wifi/master/install.sh</span>{' '}
                <span className="text-purple-400 font-bold">|</span>{' '}
                <span className="text-emerald-400 font-semibold">bash</span>
              </CommandBlock>
            </div>
          </div>
        </SpotlightCard>
      </div>
    </section>
  );
};
