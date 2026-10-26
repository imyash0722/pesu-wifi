import React, { useState, useEffect } from 'react';
import { Wifi, ShieldCheck, Zap, UserCheck, RefreshCw, AlertTriangle, ArrowRight, Activity } from 'lucide-react';

export const InteractiveSimulator: React.FC = () => {
  const [isConnected, setIsConnected] = useState(true);
  const [activeAccount, setActiveAccount] = useState<'PES1UG22CS001' | 'PES1UG22CS999'>('PES1UG22CS001');
  const [vpnActive, setVpnActive] = useState(false);
  const [latency, setLatency] = useState(14);
  const [logs, setLogs] = useState<string[]>([
    '[12:00:00] AP: PESU-EC-Campus (BSSID 00:27:0d:3f:14:e0) RSSI -58 dBm',
    '[12:00:01] Gateway: http://192.168.254.1:8090/live -> HTTP 200 <ack>ack</ack>',
    '[12:00:02] Session status: Authenticated (User PES1UG22CS001)'
  ]);
  const [isPinging, setIsPinging] = useState(false);

  const addLog = (msg: string) => {
    const time = new Date().toTimeString().split(' ')[0];
    setLogs((prev) => [`[${time}] ${msg}`, ...prev.slice(0, 4)]);
  };

  const handlePing = () => {
    setIsPinging(true);
    addLog(`DCD Keepalive heartbeat dispatch (120s ±30s jittered)`);
    setTimeout(() => {
      const ms = Math.floor(Math.random() * 8) + 11;
      setLatency(ms);
      addLog(`Gateway ACK received: <liverequestresponse><ack>ack</ack></liverequestresponse> (${ms}ms)`);
      setIsPinging(false);
    }, 450);
  };

  const handleSimulateRoaming = () => {
    addLog('Physical Wi-Fi Handover detected: RSSI drop -> BSSID 00:27:0d:3f:14:f0');
    addLog('Acquiring 20s CPU WakeLock for LinkProperties DHCP renewal...');
    setTimeout(() => {
      addLog('Zero-latency re-auth (<150ms): Session acknowledged by campus router.');
    }, 300);
  };

  const handleSimulateFailover = () => {
    addLog('Account PES1UG22CS001: Gateway returned "Maximum Login Limit Reached"');
    addLog('⚡ Multi-Account Auto-Failover engaged: switching to backup account PES1UG22CS999');
    setTimeout(() => {
      setActiveAccount('PES1UG22CS999');
      addLog('Successfully authenticated with PES1UG22CS999. Connectivity preserved.');
    }, 500);
  };

  const handleToggleVpn = () => {
    const next = !vpnActive;
    setVpnActive(next);
    if (next) {
      addLog('Active VPN tunnel detected (TRANSPORT_VPN).');
      addLog('ResilientSocketFactory: socket.connect() caught kernel EPERM -> seamlessly bypassing via local RFC-1918 gateway.');
    } else {
      addLog('VPN disconnected: reverting to standard Wi-Fi socket factory.');
    }
  };

  return (
    <div className="rounded-2xl border border-white/10 bg-[#0d0f17] p-6 lg:p-8 shadow-2xl">
      <div className="flex flex-col md:flex-row items-start md:items-center justify-between pb-6 border-b border-white/5 gap-4">
        <div>
          <div className="flex items-center gap-2">
            <span className="flex h-2.5 w-2.5 rounded-full bg-emerald-400 animate-pulse" />
            <span className="text-xs font-semibold uppercase tracking-wider text-emerald-400">Live Campus Engine</span>
          </div>
          <h3 className="text-xl font-bold text-white mt-1">Interactive Keepalive & Roaming Simulator</h3>
          <p className="text-xs text-slate-400">Interact with the real-time background state machine</p>
        </div>

        {/* State Badges */}
        <div className="flex flex-wrap items-center gap-2">
          <div className={`px-3 py-1.5 rounded-full text-xs font-medium flex items-center gap-1.5 ${
            isConnected ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/30' : 'bg-red-500/10 text-red-400 border border-red-500/30'
          }`}>
            <Wifi className="w-3.5 h-3.5" />
            <span>{isConnected ? 'Authenticated' : 'Disconnected'}</span>
          </div>

          {vpnActive && (
            <div className="px-3 py-1.5 rounded-full text-xs font-medium bg-blue-500/10 text-blue-400 border border-blue-500/30 flex items-center gap-1.5">
              <ShieldCheck className="w-3.5 h-3.5" />
              <span>VPN Resilient</span>
            </div>
          )}

          <div className="px-3 py-1.5 rounded-full text-xs font-medium bg-slate-800 text-slate-300 border border-white/10 flex items-center gap-1.5">
            <Activity className="w-3.5 h-3.5 text-emerald-400" />
            <span>{latency} ms</span>
          </div>
        </div>
      </div>

      {/* Simulator Controls & Telemetry */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mt-6">
        {/* Controls */}
        <div className="flex flex-col gap-3">
          <span className="text-xs font-medium text-slate-400 uppercase tracking-wider">Simulate Engine Actions</span>
          
          <button
            onClick={handlePing}
            disabled={isPinging}
            className="flex items-center justify-between px-4 py-3 rounded-xl bg-white/5 hover:bg-white/10 border border-white/5 hover:border-emerald-500/40 text-left transition-all group"
          >
            <div className="flex items-center gap-3">
              <RefreshCw className={`w-4 h-4 text-emerald-400 ${isPinging ? 'animate-spin' : 'group-hover:rotate-45'} transition-transform`} />
              <div>
                <div className="text-sm font-semibold text-slate-200">Send 120s DCD Keepalive</div>
                <div className="text-xs text-slate-400">Dispatches jittered XML heartbeat to router</div>
              </div>
            </div>
            <ArrowRight className="w-4 h-4 text-slate-500 group-hover:text-emerald-400 group-hover:translate-x-1 transition-all" />
          </button>

          <button
            onClick={handleSimulateRoaming}
            className="flex items-center justify-between px-4 py-3 rounded-xl bg-white/5 hover:bg-white/10 border border-white/5 hover:border-blue-500/40 text-left transition-all group"
          >
            <div className="flex items-center gap-3">
              <Zap className="w-4 h-4 text-blue-400 group-hover:scale-110 transition-transform" />
              <div>
                <div className="text-sm font-semibold text-slate-200">Simulate Hallway AP Roam</div>
                <div className="text-xs text-slate-400">Tests 20s WakeLock & &lt;150ms re-association</div>
              </div>
            </div>
            <ArrowRight className="w-4 h-4 text-slate-500 group-hover:text-blue-400 group-hover:translate-x-1 transition-all" />
          </button>

          <button
            onClick={handleSimulateFailover}
            className="flex items-center justify-between px-4 py-3 rounded-xl bg-white/5 hover:bg-white/10 border border-white/5 hover:border-amber-500/40 text-left transition-all group"
          >
            <div className="flex items-center gap-3">
              <UserCheck className="w-4 h-4 text-amber-400 group-hover:scale-110 transition-transform" />
              <div>
                <div className="text-sm font-semibold text-slate-200">Simulate Limit Hit & Auto-Failover</div>
                <div className="text-xs text-slate-400">Instant fallback to saved backup account</div>
              </div>
            </div>
            <ArrowRight className="w-4 h-4 text-slate-500 group-hover:text-amber-400 group-hover:translate-x-1 transition-all" />
          </button>

          <button
            onClick={handleToggleVpn}
            className={`flex items-center justify-between px-4 py-3 rounded-xl border text-left transition-all group ${
              vpnActive 
                ? 'bg-blue-500/10 border-blue-500/40 text-blue-300' 
                : 'bg-white/5 hover:bg-white/10 border-white/5 hover:border-slate-500/40 text-slate-300'
            }`}
          >
            <div className="flex items-center gap-3">
              <ShieldCheck className={`w-4 h-4 ${vpnActive ? 'text-blue-400' : 'text-slate-400'}`} />
              <div>
                <div className="text-sm font-semibold">{vpnActive ? 'VPN Active: EPERM Bypassed' : 'Toggle Active VPN'}</div>
                <div className="text-xs text-slate-400">Tailscale, WireGuard, WARP, OpenVPN test</div>
              </div>
            </div>
            <span className="text-xs font-mono px-2 py-0.5 rounded bg-black/40">{vpnActive ? 'ON' : 'OFF'}</span>
          </button>
        </div>

        {/* Live Terminal Logs */}
        <div className="flex flex-col">
          <div className="flex items-center justify-between mb-2">
            <span className="text-xs font-medium text-slate-400 uppercase tracking-wider">Engine Telemetry Feed</span>
            <span className="text-[10px] font-mono text-slate-500">Auto-Refreshed</span>
          </div>
          <div className="flex-1 rounded-xl bg-black/60 border border-white/5 p-4 font-mono text-xs overflow-hidden flex flex-col justify-end">
            <div className="space-y-2 text-slate-300">
              {logs.map((log, i) => (
                <div
                  key={i}
                  className={`leading-relaxed transition-all duration-300 ${
                    i === 0 ? 'text-emerald-400 font-semibold' : 'text-slate-400 opacity-80'
                  }`}
                >
                  {log}
                </div>
              ))}
            </div>
            <div className="flex items-center gap-2 mt-4 pt-3 border-t border-white/5 text-[11px] text-slate-500">
              <div className="h-1.5 w-1.5 rounded-full bg-emerald-500 animate-ping" />
              <span>Active User: <strong className="text-slate-300">{activeAccount}</strong></span>
              <span className="mx-1">•</span>
              <span>Port: <strong className="text-slate-300">8090</strong></span>
              <span className="mx-1">•</span>
              <span>Watchdog: <strong className="text-emerald-400">Armed</strong></span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
