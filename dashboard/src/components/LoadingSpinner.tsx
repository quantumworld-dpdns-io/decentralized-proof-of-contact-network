'use client';

export default function LoadingSpinner({ size = 'md', text }: { size?: 'sm' | 'md' | 'lg'; text?: string }) {
  const sizeMap = { sm: 'h-4 w-4', md: 'h-8 w-8', lg: 'h-12 w-12' };
  return (
    <div className="flex flex-col items-center justify-center gap-3 py-12">
      <div className={`${sizeMap[size]} animate-spin rounded-full border-2 border-surface-300 dark:border-surface-600 border-t-primary-600`} />
      {text && <p className="text-sm text-surface-500 dark:text-surface-400">{text}</p>}
    </div>
  );
}
