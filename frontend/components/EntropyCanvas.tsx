'use client';

import { useEffect, useRef } from 'react';
import { useEntropyStore } from '@/store/entropyStore';
import { entropyPalette } from '@/styles/palette';

type Particle = {
  angle: number;
  orbit: number;
  speed: number;
  depth: number;
  phase: number;
};

const PARTICLE_COUNT = 150;

export function EntropyCanvas() {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    const context = canvas?.getContext('2d', { alpha: true });

    if (!canvas || !context) {
      return undefined;
    }

    let frame = 0;
    let width = 0;
    let height = 0;
    let intensity = useEntropyStore.getState().entropy.intensity;
    let stability = useEntropyStore.getState().entropy.stability;
    let hasEntropySignal = useEntropyStore.getState().hasEntropySignal;
    let targetIntensity = intensity;
    let targetStability = stability;

    const particles = createParticles();
    const unsubscribe = useEntropyStore.subscribe((state) => {
      targetIntensity = state.entropy.intensity;
      targetStability = state.entropy.stability;
      hasEntropySignal = state.hasEntropySignal;
    });

    const resize = () => {
      const rect = canvas.getBoundingClientRect();
      const nextWidth = Math.max(1, rect.width);
      const nextHeight = Math.max(1, rect.height);
      const ratio = Math.min(window.devicePixelRatio || 1, 2);

      width = nextWidth;
      height = nextHeight;
      canvas.width = Math.floor(nextWidth * ratio);
      canvas.height = Math.floor(nextHeight * ratio);
      context.setTransform(ratio, 0, 0, ratio, 0, 0);
    };

    const observer = new ResizeObserver(resize);
    observer.observe(canvas);
    resize();

    const render = (time: number) => {
      intensity += (targetIntensity - intensity) * 0.035;
      stability += (targetStability - stability) * 0.035;

      if (hasEntropySignal) {
        drawField(context, particles, width, height, time / 1000, intensity, stability);
      } else {
        drawDormantField(context, width, height);
      }

      frame = requestAnimationFrame(render);
    };

    frame = requestAnimationFrame(render);

    return () => {
      cancelAnimationFrame(frame);
      observer.disconnect();
      unsubscribe();
    };
  }, []);

  return (
    <canvas
      ref={canvasRef}
      aria-label="Realtime entropy field visualization"
      className="absolute inset-0 h-full w-full"
    />
  );
}

function drawDormantField(context: CanvasRenderingContext2D, width: number, height: number) {
  const centerX = width / 2;
  const centerY = height / 2;
  const radius = Math.min(width, height);

  context.globalCompositeOperation = 'source-over';
  context.fillStyle = 'rgba(5, 6, 7, 1)';
  context.fillRect(0, 0, width, height);

  const gradient = context.createRadialGradient(centerX, centerY, radius * 0.04, centerX, centerY, radius * 0.72);
  gradient.addColorStop(0, `rgba(${entropyPalette.emerald}, 0.05)`);
  gradient.addColorStop(0.54, `rgba(${entropyPalette.cyan}, 0.025)`);
  gradient.addColorStop(1, 'rgba(5, 6, 7, 0)');
  context.fillStyle = gradient;
  context.fillRect(0, 0, width, height);

  context.strokeStyle = 'rgba(255, 255, 255, 0.045)';
  context.lineWidth = 1;
  context.beginPath();
  context.ellipse(centerX, centerY, radius * 0.34, radius * 0.21, 0, 0, Math.PI * 2);
  context.stroke();
}

function createParticles(): Particle[] {
  return Array.from({ length: PARTICLE_COUNT }, (_, index) => {
    const lane = index / PARTICLE_COUNT;

    return {
      angle: lane * Math.PI * 2,
      orbit: 0.12 + (index % 21) / 24,
      speed: 0.16 + ((index * 17) % 31) / 70,
      depth: 0.3 + ((index * 13) % 67) / 100,
      phase: ((index * 29) % 360) * (Math.PI / 180)
    };
  });
}

function drawField(
  context: CanvasRenderingContext2D,
  particles: Particle[],
  width: number,
  height: number,
  time: number,
  intensity: number,
  stability: number
) {
  const centerX = width / 2;
  const centerY = height / 2;
  const radius = Math.min(width, height);
  const instability = 1 - stability;

  context.globalCompositeOperation = 'source-over';
  context.fillStyle = `rgba(5, 6, 7, ${0.24 + instability * 0.18})`;
  context.fillRect(0, 0, width, height);

  const gradient = context.createRadialGradient(centerX, centerY, radius * 0.02, centerX, centerY, radius * 0.72);
  gradient.addColorStop(0, `rgba(${entropyPalette.emerald}, ${0.16 + intensity * 0.16})`);
  gradient.addColorStop(0.42, `rgba(${entropyPalette.cyan}, ${0.05 + stability * 0.08})`);
  gradient.addColorStop(0.72, `rgba(${entropyPalette.amber}, ${0.025 + intensity * 0.035})`);
  gradient.addColorStop(1, 'rgba(5, 6, 7, 0)');
  context.fillStyle = gradient;
  context.fillRect(0, 0, width, height);

  context.save();
  context.globalCompositeOperation = 'lighter';
  drawRibbons(context, centerX, centerY, radius, time, intensity, stability);
  drawParticles(context, particles, centerX, centerY, radius, time, intensity, stability);
  context.restore();
}

function drawRibbons(
  context: CanvasRenderingContext2D,
  centerX: number,
  centerY: number,
  radius: number,
  time: number,
  intensity: number,
  stability: number
) {
  const ribbonCount = 9;

  for (let ribbon = 0; ribbon < ribbonCount; ribbon++) {
    const phase = ribbon * 0.71;
    const orbit = radius * (0.18 + ribbon * 0.035 + intensity * 0.035);

    context.beginPath();

    for (let step = 0; step <= 180; step++) {
      const t = step / 180;
      const angle = t * Math.PI * 2;
      const pulse = Math.sin(angle * (2.2 + stability) + time * intensity * 1.55 + phase);
      const fold = Math.cos(angle * 3.1 - time * (1 - stability) * 0.72 + phase);
      const localRadius = orbit + pulse * radius * 0.032 + fold * radius * (0.01 + (1 - stability) * 0.026);
      const x = centerX + Math.cos(angle + time * intensity * 0.075 + phase * 0.04) * localRadius;
      const y = centerY + Math.sin(angle - time * intensity * 0.06 + phase * 0.02) * localRadius * 0.72;

      if (step === 0) {
        context.moveTo(x, y);
      } else {
        context.lineTo(x, y);
      }
    }

    const alpha = 0.035 + intensity * 0.035 + ribbon * 0.002;
    context.lineWidth = 0.55 + intensity * 1.2;
    context.strokeStyle =
      ribbon % 3 === 0
        ? `rgba(${entropyPalette.emerald}, ${alpha})`
        : ribbon % 3 === 1
          ? `rgba(${entropyPalette.cyan}, ${alpha * 0.82})`
          : `rgba(${entropyPalette.amber}, ${alpha * 0.72})`;
    context.stroke();
  }
}

function drawParticles(
  context: CanvasRenderingContext2D,
  particles: Particle[],
  centerX: number,
  centerY: number,
  radius: number,
  time: number,
  intensity: number,
  stability: number
) {
  const instability = 1 - stability;

  for (const particle of particles) {
    const drift = time * particle.speed * intensity * 1.12;
    const angle = particle.angle + drift + Math.sin(time * intensity * 0.9 + particle.phase) * instability * 0.85;
    const orbit = radius * particle.orbit * (0.5 + particle.depth * 0.85);
    const turbulence = Math.sin(time * intensity * 1.4 + particle.phase) * radius * instability * 0.035;
    const x = centerX + Math.cos(angle) * (orbit + turbulence);
    const y = centerY + Math.sin(angle * (1.05 + instability * 0.1)) * (orbit * 0.66) + turbulence * 0.45;
    const particleRadius = 0.55 + particle.depth * 1.6 + intensity * 1.7;
    const alpha = 0.18 + intensity * 0.46 - particle.depth * 0.08;

    context.beginPath();
    context.arc(x, y, particleRadius, 0, Math.PI * 2);
    context.fillStyle =
      particle.depth > 0.72
        ? `rgba(${entropyPalette.amber}, ${alpha * 0.82})`
        : particle.depth > 0.48
          ? `rgba(${entropyPalette.cyan}, ${alpha})`
          : `rgba(${entropyPalette.emerald}, ${alpha})`;
    context.fill();
  }
}
