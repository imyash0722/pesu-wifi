import React from 'react';

interface ShinyButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  children: React.ReactNode;
  variant?: 'primary' | 'secondary' | 'outline';
  className?: string;
  icon?: React.ReactNode;
  rightIcon?: React.ReactNode;
}

export const ShinyButton: React.FC<ShinyButtonProps> = ({
  children,
  variant = 'primary',
  className = '',
  icon,
  rightIcon,
  ...props
}) => {
  const baseClasses =
    'relative group overflow-hidden rounded-xl font-medium text-sm transition-all duration-300 active:scale-[0.98] flex items-center gap-3 cursor-pointer whitespace-nowrap select-none';

  const variants = {
    primary:
      'bg-gradient-to-r from-emerald-500 to-emerald-600 text-white shadow-lg shadow-emerald-500/20 hover:shadow-emerald-500/40 hover:brightness-110 px-5 py-3.5 justify-center',
    secondary:
      'bg-[#1a1e29] text-slate-200 border border-white/10 hover:border-white/20 hover:bg-[#222735] px-5 py-3.5 shadow-md justify-center',
    outline:
      'bg-transparent text-slate-300 border border-white/10 hover:border-purple-500/50 hover:text-purple-300 px-5 py-3.5 justify-center',
  };

  return (
    <button className={`${baseClasses} ${variants[variant]} ${className}`} {...props}>
      {/* Animated Shimmer Overlay */}
      <span className="absolute inset-0 w-full h-full transform -translate-x-full group-hover:translate-x-full transition-transform duration-1000 bg-gradient-to-r from-transparent via-white/15 to-transparent pointer-events-none" />

      {icon && <span className="relative z-10 shrink-0 flex items-center">{icon}</span>}
      <span className={`relative z-10 truncate ${rightIcon ? 'flex-1 text-left' : ''}`}>{children}</span>
      {rightIcon && <span className="relative z-10 shrink-0 flex items-center ml-auto">{rightIcon}</span>}
    </button>
  );
};
