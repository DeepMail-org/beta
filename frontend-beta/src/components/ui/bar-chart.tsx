"use client"

import { Bar, BarChart as RechartsBarChart, CartesianGrid, XAxis, YAxis, Cell } from "recharts"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { ChartConfig, ChartContainer, ChartTooltip, ChartTooltipContent } from "@/components/ui/chart"

const chartConfig = {
  count: {
    label: "Count",
    color: "hsl(var(--accent))",
  },
} satisfies ChartConfig

export function BarChart({
  data,
  title = "Top Suspicious Senders",
  subtitle = "Past 24 hours",
  className,
}: {
  data: { label: string; value: number }[];
  title?: string;
  subtitle?: string;
  className?: string;
}) {
  return (
    <Card className={`flex flex-col h-full w-full ${className || ''}`}>
      <CardHeader className="pb-4 bg-surface/50 border-b border-border">
        <CardTitle className="text-sm font-display font-semibold tracking-wide text-foreground">{title}</CardTitle>
        <CardDescription className="text-xs text-muted mt-1">{subtitle}</CardDescription>
      </CardHeader>
      <CardContent className="pb-4 pt-6 flex-1 flex flex-col justify-center min-h-[200px]">
        <ChartContainer
          config={chartConfig}
          className="w-full h-[220px]"
        >
          <RechartsBarChart
            data={data}
            layout="vertical"
            margin={{ top: 0, right: 10, bottom: 0, left: -20 }}
          >
            <CartesianGrid horizontal={true} vertical={false} strokeDasharray="3 3" stroke="var(--color-border)" opacity={0.5} />
            <XAxis type="number" hide />
            <YAxis 
              dataKey="label" 
              type="category" 
              axisLine={false} 
              tickLine={false} 
              tick={{ fill: 'var(--color-foreground)', fontSize: 12, fontWeight: 500 }}
              width={200}
            />
            <ChartTooltip cursor={{ fill: 'var(--color-surface-2)' }} content={<ChartTooltipContent hideLabel />} />
            <Bar dataKey="value" radius={[0, 4, 4, 0]} barSize={24}>
              {data.map((entry, index) => (
                <Cell key={`cell-${index}`} fill={`hsl(var(--accent) / ${1 - index * 0.15})`} />
              ))}
            </Bar>
          </RechartsBarChart>
        </ChartContainer>
      </CardContent>
    </Card>
  )
}
