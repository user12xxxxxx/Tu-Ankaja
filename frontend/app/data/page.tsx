'use client';

import { useEffect, useMemo } from 'react';
import { motion } from 'framer-motion';
import { Radio, Hash, Activity, Thermometer, Move3D, Palette, Sun, Wind, Zap } from 'lucide-react';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';
import { getNumbers, getParams } from '@/services/otpService';
import { useOtpStore } from '@/store/otpStore';

// Fixed CSV column headers from hardware (21 columns)
const COLUMNS = [
  'MOSFET_Noise',
  'Temp(°C)',
  'AccX(m/s²)', 'AccY(m/s²)', 'AccZ(m/s²)',
  'GyroX(°/s)', 'GyroY(°/s)', 'GyroZ(°/s)',
  'Red(raw)', 'Green(raw)', 'Blue(raw)',
  'AmbientLight(Lux)',
  'PM1.0(µg/m³)', 'PM2.5(µg/m³)', 'PM10(µg/m³)',
  'Count>0.3µm', 'Count>0.5µm', 'Count>1.0µm',
  'Count>2.5µm', 'Count>5.0µm', 'Count>10µm',
];

// Group sensors by category for separate charts (different scales)
const CHART_GROUPS = [
  {
    title: 'MOSFET Noise',
    icon: Zap,
    keys: ['MOSFET_Noise'],
    colors: ['#e879f9'],
  },
  {
    title: 'Color Sensor (RGB)',
    icon: Palette,
    keys: ['Red(raw)', 'Green(raw)', 'Blue(raw)'],
    colors: ['#ef4444', '#22c55e', '#3b82f6'],
  },
  {
    title: 'Ambient Light',
    icon: Sun,
    keys: ['AmbientLight(Lux)'],
    colors: ['#facc15'],
  },
  {
    title: 'Temperature',
    icon: Thermometer,
    keys: ['Temp(°C)'],
    colors: ['#f97316'],
  },
  {
    title: 'Accelerometer',
    icon: Move3D,
    keys: ['AccX(m/s²)', 'AccY(m/s²)', 'AccZ(m/s²)'],
    colors: ['#34d399', '#22d3ee', '#a78bfa'],
  },
  {
    title: 'Gyroscope',
    icon: Move3D,
    keys: ['GyroX(°/s)', 'GyroY(°/s)', 'GyroZ(°/s)'],
    colors: ['#fb923c', '#f472b6', '#facc15'],
  },
  {
    title: 'Particle Concentration (per 0.1L)',
    icon: Activity,
    keys: ['Count>0.3µm', 'Count>0.5µm', 'Count>1.0µm', 'Count>2.5µm', 'Count>5.0µm', 'Count>10µm'],
    colors: ['#34d399', '#22d3ee', '#a78bfa', '#fb923c', '#f472b6', '#facc15'],
  },
  {
    title: 'Mass Concentration (µg/m³)',
    icon: Wind,
    keys: ['PM1.0(µg/m³)', 'PM2.5(µg/m³)', 'PM10(µg/m³)'],
    colors: ['#60a5fa', '#f472b6', '#facc15'],
  },
];

type ChartPoint = Record<string, number | string>;

/** Parse a CSV row of numbers using the fixed column headers */
function parseCsvRow(raw: string): Record<string, number> | null {
  const values = raw.split(',').map((v) => v.trim());
  // Skip if this looks like a header row (contains letters in first field)
  if (values.length > 0 && /[a-zA-Z]/.test(values[0])) {
    return null;
  }
  if (values.length < COLUMNS.length) {
    return null;
  }
  const result: Record<string, number> = {};
  for (let i = 0; i < COLUMNS.length; i++) {
    const num = parseFloat(values[i]);
    if (Number.isFinite(num)) {
      result[COLUMNS[i]] = num;
    }
  }
  return Object.keys(result).length > 0 ? result : null;
}

const TOOLTIP_STYLE = {
  backgroundColor: '#141618',
  border: '1px solid rgba(255,255,255,0.1)',
  borderRadius: '8px',
  fontSize: '11px',
  color: '#e5e5e5',
};

export default function DataPage() {
  const numbers = useOtpStore((s) => s.numbers);
  const params = useOtpStore((s) => s.params);
  const setNumbers = useOtpStore((s) => s.setNumbers);
  const setParams = useOtpStore((s) => s.setParams);

  useEffect(() => {
    const fetchAll = () => {
      getNumbers().then((d) => setNumbers(d.numbers)).catch(() => {});
      getParams().then((d) => setParams(d.params)).catch(() => {});
    };
    fetchAll();
    const timer = setInterval(fetchAll, 3000);
    return () => clearInterval(timer);
  }, [setNumbers, setParams]);

  // Parse all CSV rows into chart data points
  const chartData = useMemo(() => {
    const points: ChartPoint[] = [];
    for (let i = 0; i < params.length; i++) {
      const parsed = parseCsvRow(params[i]);
      if (parsed) {
        points.push({ index: points.length + 1, ...parsed });
      }
    }
    return points.slice(-100);
  }, [params]);

  const hasData = chartData.length > 0;

  return (
    <div className="mx-auto max-w-5xl px-4 py-8">
      <div className="mb-6">
        <h1 className="text-2xl font-semibold text-neutral-100">Raw Data Viewer</h1>
        <p className="mt-1 text-sm text-neutral-500">
          Live MQTT sensor data — 21 parameters across 8 sensor groups
        </p>
        {hasData && (
          <p className="mt-0.5 text-[10px] text-neutral-600">
            {chartData.length} samples received
          </p>
        )}
      </div>

      {/* Sensor Charts — grouped by category */}
      <div className="mb-6 grid gap-4 md:grid-cols-2">
        {CHART_GROUPS.map((group) => {
          const Icon = group.icon;
          return (
            <section
              key={group.title}
              className="rounded-xl border border-white/[0.07] bg-white/[0.03] backdrop-blur-xl"
            >
              <div className="flex items-center gap-2 border-b border-white/[0.07] px-4 py-2.5">
                <Icon className="h-3.5 w-3.5 text-cyan-400" />
                <h2 className="text-xs font-medium text-neutral-200">{group.title}</h2>
                <span className="ml-auto text-[10px] text-neutral-600">
                  {group.keys.join(', ')}
                </span>
              </div>

              <div className="p-3">
                {hasData ? (
                  <ResponsiveContainer width="100%" height={180}>
                    <LineChart data={chartData}>
                      <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.05)" />
                      <XAxis
                        dataKey="index"
                        tick={{ fontSize: 9, fill: '#525252' }}
                        stroke="rgba(255,255,255,0.07)"
                      />
                      <YAxis
                        tick={{ fontSize: 9, fill: '#525252' }}
                        stroke="rgba(255,255,255,0.07)"
                        width={45}
                      />
                      <Tooltip contentStyle={TOOLTIP_STYLE} />
                      <Legend wrapperStyle={{ fontSize: '10px', color: '#737373' }} />
                      {group.keys.map((key, i) => (
                        <Line
                          key={key}
                          type="monotone"
                          dataKey={key}
                          stroke={group.colors[i]}
                          strokeWidth={1.5}
                          dot={{ r: 2, strokeWidth: 1 }}
                          activeDot={{ r: 4 }}
                        />
                      ))}
                    </LineChart>
                  </ResponsiveContainer>
                ) : (
                  <div className="flex h-[180px] items-center justify-center text-[11px] text-neutral-600">
                    Waiting for sensor data...
                  </div>
                )}
              </div>
            </section>
          );
        })}
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        {/* Sensor Parameters Raw Data */}
        <section className="flex flex-col rounded-xl border border-white/[0.07] bg-white/[0.03] backdrop-blur-xl">
          <div className="flex items-center gap-2 border-b border-white/[0.07] px-4 py-3">
            <Radio className="h-4 w-4 text-cyan-400" />
            <h2 className="text-sm font-medium text-neutral-200">Raw Sensor Data</h2>
            <span className="ml-auto text-[10px] text-neutral-600">
              {params.length} entries
            </span>
          </div>

          <div className="max-h-[40vh] min-h-[200px] overflow-y-auto p-2 scrollbar-hide">
            {params.length > 0 ? (
              <div className="space-y-1">
                {[...params].reverse().map((param, i) => (
                  <motion.div
                    key={`param-${i}`}
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    className="rounded-md bg-white/[0.02] px-3 py-1.5 font-mono text-xs text-neutral-400 break-all"
                  >
                    {param}
                  </motion.div>
                ))}
              </div>
            ) : (
              <div className="flex h-full min-h-[180px] items-center justify-center text-xs text-neutral-600">
                Waiting for data on random/params...
              </div>
            )}
          </div>
        </section>

        {/* Random Numbers */}
        <section className="flex flex-col rounded-xl border border-white/[0.07] bg-white/[0.03] backdrop-blur-xl">
          <div className="flex items-center gap-2 border-b border-white/[0.07] px-4 py-3">
            <Hash className="h-4 w-4 text-emerald-400" />
            <h2 className="text-sm font-medium text-neutral-200">4-Digit Random Numbers</h2>
            <span className="ml-auto text-[10px] text-neutral-600">
              {numbers.length} numbers
            </span>
          </div>

          <div className="max-h-[40vh] min-h-[200px] overflow-y-auto p-3 scrollbar-hide">
            {numbers.length > 0 ? (
              <div className="flex flex-wrap gap-1.5">
                {[...numbers].reverse().map((num, i) => (
                  <span
                    key={`num-${i}`}
                    className="inline-block rounded border border-white/[0.05] bg-white/[0.02] px-2 py-0.5 font-mono text-xs text-emerald-300/80"
                  >
                    {String(num).padStart(4, '0')}
                  </span>
                ))}
              </div>
            ) : (
              <div className="flex h-full min-h-[180px] items-center justify-center text-xs text-neutral-600">
                Waiting for data on random/numbers...
              </div>
            )}
          </div>
        </section>
      </div>
    </div>
  );
}
