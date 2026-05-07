"use client"

import * as React from "react"
import { CartesianGrid, Line, LineChart as RechartsLineChart, XAxis } from "recharts"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { ChartConfig, ChartContainer, ChartTooltip, ChartTooltipContent } from "@/components/ui/chart"
import { Tag } from "@/components/dashboard/primitives"
import { Btn } from "@/components/dashboard/primitives"

const chartConfig = {
  value: {
    label: "Value",
    color: "hsl(var(--accent))",
  },
} satisfies ChartConfig

export function LineChartComponent({
  data,
  title = "Threat Timeline",
  subtitle = "Detections over the last 7 days",
  className,
}: {
  data: { label: string; value: number }[];
  title?: string;
  subtitle?: string;
  className?: string;
}) {
  const [period, setPeriod] = React.useState("7D");

  return (
    <Card className={`flex flex-col h-full ${className || ''}`}>
      <CardHeader className="flex flex-row items-center justify-between pb-2 bg-surface/50 border-b border-border">
        <div>
          <CardTitle className="text-sm font-display font-semibold tracking-wide text-foreground">{title}</CardTitle>
          <CardDescription className="text-xs text-muted mt-1">{subtitle}</CardDescription>
        </div>
        <div className="flex items-center gap-2">
          <div className="flex bg-glass border border-border rounded-md p-0.5">
            {["1D", "7D", "1M", "3M"].map(p => (
              <button
                key={p}
                onClick={() => setPeriod(p)}
                className={`px-3 py-1 text-xs font-medium rounded-sm transition-colors ${period === p ? 'bg-glass-strong text-foreground shadow-sm' : 'text-muted hover:text-secondary'}`}
              >
                {p}
              </button>
            ))}
          </div>
          <Btn size="sm">Export</Btn>
        </div>
      </CardHeader>
      <CardContent className="pb-4 pt-6 flex-1 flex flex-col justify-center min-h-[220px]">
        <ChartContainer
          config={chartConfig}
          className="w-full h-[220px]"
        >
          <RechartsLineChart data={data} margin={{ top: 10, right: 10, bottom: 0, left: 10 }}>
            <CartesianGrid vertical={false} strokeDasharray="3 3" stroke="var(--color-border)" opacity={0.5} />
            <XAxis dataKey="label" tickLine={false} axisLine={false} tickMargin={8} tick={{ fill: 'var(--color-muted)' }} />
            <ChartTooltip cursor={false} content={<ChartTooltipContent />} />
            <Line
              type="monotone"
              dataKey="value"
              stroke="var(--color-accent, #8b5cf6)"
              strokeWidth={2}
              dot={{ r: 3, fill: 'var(--color-background)', strokeWidth: 2 }}
              activeDot={{ r: 5 }}
            />
          </RechartsLineChart>
        </ChartContainer>
      </CardContent>
    </Card>
  )
}
