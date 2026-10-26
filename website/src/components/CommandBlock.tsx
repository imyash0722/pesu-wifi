import React, { useState } from 'react';
import { Copy, Check } from 'lucide-react';

interface CommandBlockProps {
  command: string;
  language?: string;
  prompt?: string;
  children: React.ReactNode;
  className?: string;
}

export const CommandBlock: React.FC<CommandBlockProps> = ({
  command,
  language = 'bash',
  prompt = '$',
  children,
  className = '',
}) => {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      if (navigator?.clipboard?.writeText) {
        await navigator.clipboard.writeText(command);
      } else {
        const textarea = document.createElement('textarea');
        textarea.value = command;
        textarea.style.position = 'fixed';
        textarea.style.opacity = '0';
        document.body.appendChild(textarea);
        textarea.select();
        document.execCommand('copy');
        document.body.removeChild(textarea);
      }
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy', err);
    }
  };

  return (
    <div
      className={`rounded-xl border border-white/10 bg-[#090c13]/95 backdrop-blur-md overflow-hidden transition-all duration-200 hover:border-white/20 shadow-lg ${className}`}
    >
      {/* Top Header Bar */}
      <div className="flex items-center justify-between px-3 py-1.5 bg-black/40 border-b border-white/5">
        <div className="flex items-center gap-2">
          {/* Terminal Dots */}
          <div className="flex items-center gap-1.5">
            <span className="w-2.5 h-2.5 rounded-full bg-red-500/70" />
            <span className="w-2.5 h-2.5 rounded-full bg-amber-500/70" />
            <span className="w-2.5 h-2.5 rounded-full bg-emerald-500/70" />
          </div>
          {/* Shell Badge */}
          <span className="ml-1 text-[10px] font-mono uppercase tracking-wider text-slate-400 bg-white/5 px-1.5 py-0.5 rounded border border-white/5">
            {language}
          </span>
        </div>

        {/* Copy Button */}
        <button
          type="button"
          onClick={handleCopy}
          className={`flex items-center gap-1.5 px-2.5 py-1 rounded-md text-[11px] font-medium transition-all duration-200 cursor-pointer active:scale-95 ${
            copied
              ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30'
              : 'bg-white/5 hover:bg-white/10 text-slate-300 hover:text-white border border-white/5'
          }`}
          title="Copy command to clipboard"
        >
          {copied ? (
            <>
              <Check className="w-3.5 h-3.5 text-emerald-400" />
              <span>Copied!</span>
            </>
          ) : (
            <>
              <Copy className="w-3.5 h-3.5 text-slate-400" />
              <span>Copy</span>
            </>
          )}
        </button>
      </div>

      {/* Code Body with Syntax Colouring */}
      <div className="px-3 py-2.5 font-mono text-[11px] leading-relaxed overflow-x-auto scrollbar-thin select-all">
        <div className="flex items-center whitespace-nowrap">
          <span className="text-slate-500 font-semibold select-none mr-2.5 shrink-0">
            {prompt}
          </span>
          <code className="text-slate-200">{children}</code>
        </div>
      </div>
    </div>
  );
};
