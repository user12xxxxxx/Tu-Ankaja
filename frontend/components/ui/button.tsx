'use client';

import * as React from 'react';
import { cva, type VariantProps } from 'class-variance-authority';
import { cn } from '@/lib/utils';

const buttonVariants = cva(
  'inline-flex h-11 items-center justify-center gap-2 whitespace-nowrap rounded-lg border border-transparent px-4 text-sm font-medium transition duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-emerald-300/60 disabled:pointer-events-none disabled:opacity-45',
  {
    variants: {
      variant: {
        default:
          'bg-emerald-300 text-neutral-950 shadow-[0_0_28px_rgba(69,239,147,0.16)] hover:bg-emerald-200',
        secondary:
          'border-white/10 bg-white/[0.055] text-neutral-100 hover:border-white/18 hover:bg-white/[0.085]',
        ghost: 'text-neutral-300 hover:bg-white/[0.06] hover:text-white'
      },
      size: {
        default: 'h-11 px-4',
        icon: 'h-9 w-9 px-0',
        sm: 'h-9 px-3 text-xs'
      }
    },
    defaultVariants: {
      variant: 'default',
      size: 'default'
    }
  }
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, ...props }, ref) => {
    return (
      <button
        className={cn(buttonVariants({ variant, size, className }))}
        ref={ref}
        {...props}
      />
    );
  }
);
Button.displayName = 'Button';

export { Button, buttonVariants };
