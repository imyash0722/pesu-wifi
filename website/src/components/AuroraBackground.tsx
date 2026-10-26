import React from 'react';

export const AuroraBackground: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  return (
    <div className="relative min-h-screen w-full bg-[#08090d] text-slate-100 overflow-hidden">
      {/* Background Gradients & Glows */}
      <div className="pointer-events-none absolute inset-0 overflow-hidden">
        {/* Subtle Grid */}
        <div 
          className="absolute inset-0 opacity-[0.03]" 
          style={{
            backgroundImage: `linear-gradient(#fff 1px, transparent 1px), linear-gradient(90deg, #fff 1px, transparent 1px)`,
            backgroundSize: '48px 48px'
          }}
        />

        {/* Ambient Glows */}
        <div className="absolute -top-40 left-1/2 -translate-x-1/2 w-[700px] h-[400px] rounded-full bg-emerald-500/10 blur-[130px]" />
        <div className="absolute top-[30%] -left-32 w-[500px] h-[500px] rounded-full bg-blue-500/8 blur-[140px]" />
        <div className="absolute top-[60%] -right-32 w-[600px] h-[600px] rounded-full bg-emerald-600/8 blur-[150px]" />
        <div className="absolute bottom-10 left-1/3 w-[500px] h-[400px] rounded-full bg-amber-500/5 blur-[120px]" />
      </div>

      <div className="relative z-10 flex flex-col min-h-screen">
        {children}
      </div>
    </div>
  );
};
