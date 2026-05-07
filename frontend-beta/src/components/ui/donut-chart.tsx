"use client"

import { Cell, Pie, PieChart } from "recharts"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { ChartConfig, ChartContainer, ChartTooltip, ChartTooltipContent } from "@/components/ui/chart"

export function DonutChart({
  data,
  title = "Severity Mix",
  subtitle = "Last 24h",
  centerLabel,
  centerValue,
  className,
}: {
  data: { label: string; value: number; color: string }[];
  title?: string;
  subtitle?: string;
  centerLabel?: string;
  centerValue?: string;
  className?: string;
}) {
  const chartConfig = data.reduce((acc, curr) => {
    acc[curr.label] = {
      label: curr.label,
      color: curr.color,
    }
    return acc
  }, {} as Record<string, any>) satisfies ChartConfig

  return (
    <Card className={`flex flex-col h-full ${className || ''}`}>
      <CardHeader className="pb-4 bg-surface/50 border-b border-border">
        <CardTitle className="text-sm font-display font-semibold tracking-wide text-foreground">{title}</CardTitle>
        <CardDescription className="text-xs text-muted mt-1">{subtitle}</CardDescription>
      </CardHeader>
      <CardContent className="pb-4 pt-6 flex-1 flex flex-col items-center justify-center">
        <div className="flex items-center gap-6 w-full max-w-[400px]">
          <div className="relative w-40 h-40">
            <ChartContainer
              config={chartConfig}
              className="w-full h-full aspect-square"
            >
              <PieChart margin={{ top: 0, right: 0, bottom: 0, left: 0 }}>
                <ChartTooltip cursor={false} content={<ChartTooltipContent hideLabel />} />
                <Pie
                  data={data}
                  dataKey="value"
                  nameKey="label"
                  innerRadius={60}
                  outerRadius={80}
                  stroke="var(--color-surface)"
                  strokeWidth={2}
                  paddingAngle={2}
                >
                  {data.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={entry.color} />
                  ))}
                </Pie>
              </PieChart>
            </ChartContainer>
            {(centerLabel || centerValue) && (
              <div className="absolute inset-0 flex flex-col items-center justify-center pointer-events-none">
                {centerValue && <span className="text-2xl font-bold">{centerValue}</span>}
                {centerLabel && <span className="text-xs text-muted font-medium uppercase tracking-widest">{centerLabel}</span>}
              </div>
            )}
          </div>
          
          {/* Custom Legend */}
          <ul className="space-y-3 flex-1 min-w-0">
            {data.map((s, i) => (
              <li key={i} className="flex items-center justify-between gap-3 text-sm">
                <div className="flex items-center gap-2 min-w-0">
                  <span
                    className="inline-block w-3 h-3 rounded-full flex-shrink-0"
                    style={{ background: s.color }}
                  />
                  <span className="text-muted truncate">{s.label}</span>
                </div>
                <span className="font-bold text-foreground">
                  {Math.round((s.value / data.reduce((acc, curr) => acc + curr.value, 0)) * 100)}%
                </span>
              </li>
            ))}
          </ul>
        </div>
      </CardContent>
    </Card>
  )
}
