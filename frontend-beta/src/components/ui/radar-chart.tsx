"use client"

import { PolarAngleAxis, PolarGrid, Radar, RadarChart as RechartsRadarChart } from "recharts"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { ChartConfig, ChartContainer, ChartTooltip, ChartTooltipContent } from "@/components/ui/chart"

const chartConfig = {
  value: {
    label: "Intensity",
    color: "hsl(var(--chart-1))",
  },
} satisfies ChartConfig

export function RadarChart({
  axes,
  title = "Attack Vector Profile",
  subtitle = "Distribution of detected threat vectors",
  className,
}: {
  axes: { label: string; value: number }[];
  title?: string;
  subtitle?: string;
  className?: string;
}) {
  return (
    <Card className={`flex flex-col h-full ${className || ''}`}>
      <CardHeader className="items-center pb-2 bg-surface/50 border-b border-border">
        <CardTitle className="text-sm font-display font-semibold tracking-wide text-foreground">{title}</CardTitle>
        <CardDescription className="text-xs text-muted mt-1">{subtitle}</CardDescription>
      </CardHeader>
      <CardContent className="pb-4 pt-6 flex-1 flex flex-col justify-center min-h-[250px]">
        <ChartContainer
          config={chartConfig}
          className="mx-auto aspect-square w-full h-[250px]"
        >
          <RechartsRadarChart data={axes} margin={{ top: 10, right: 10, bottom: 10, left: 10 }}>
            <ChartTooltip cursor={false} content={<ChartTooltipContent />} />
            <PolarAngleAxis dataKey="label" tick={{ fill: 'var(--color-muted)', fontSize: 11 }} />
            <PolarGrid gridType="polygon" stroke="var(--color-border)" />
            <Radar
              dataKey="value"
              fill="var(--color-accent)"
              fillOpacity={0.6}
              stroke="var(--color-accent)"
              strokeWidth={2}
              dot={{
                r: 4,
                fill: "var(--color-background)",
                stroke: "var(--color-accent)",
                strokeWidth: 2,
                fillOpacity: 1,
              }}
            />
          </RechartsRadarChart>
        </ChartContainer>
      </CardContent>
    </Card>
  )
}
