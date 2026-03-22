# @c9up/technical-indicators-napi

High-performance technical analysis library written in Rust, compiled to native Node.js addon via [NAPI-RS](https://napi.rs). Provides 15 indicators and 2 chart types with zero JavaScript overhead.

[![npm version][npm-image]][npm-url]

[npm-image]: https://img.shields.io/npm/v/@c9up/technical-indicators-napi.svg?style=flat-square
[npm-url]: https://npmjs.org/package/@c9up/technical-indicators-napi

## Installation

```bash
npm install @c9up/technical-indicators-napi
```

Prebuilt binaries are available for:
- Linux x64 (glibc & musl)
- Windows x64

## Quick Start

```javascript
import {
  simpleMovingAverage,
  relativeStrengthIndex,
  bollingerBands,
  ichimoku,
} from '@c9up/technical-indicators-napi'

// Simple price array indicators
const sma = simpleMovingAverage([1, 2, 3, 4, 5, 6, 7, 8, 9, 10], 3)
const rsi = relativeStrengthIndex([44, 44.34, 44.09, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84], 5)

// OHLCV indicators require MarketData objects
const marketData = [
  { open: 100, high: 105, low: 98, close: 103, volume: 1000, date: '2024-01-01' },
  { open: 103, high: 107, low: 101, close: 106, volume: 1200, date: '2024-01-02' },
  // ...
]
const cloud = ichimoku(marketData)
```

## Input Types

### Price Array

Simple `Array<number>` of closing prices. Used by:
`simpleMovingAverage`, `exponentialMovingAverage`, `relativeStrengthIndex`, `bollingerBands`, `extractImportantLevels`, `renkoChart`, `kagiChart`

### MarketData

OHLCV object required by most advanced indicators:

```typescript
interface MarketData {
  low: number
  high: number
  open: number
  close: number
  volume: number
  date: string
}
```

Used by: `directionalMovementIndex`, `parabolicSar`, `stochasticOscillator`, `stochasticMomentumIndex`, `ichimoku`, `trendsMeter`, `pivotPoints`, `entryExitSignals`, `kReversal`

---

## Indicators

### Simple Moving Average (SMA)

```typescript
simpleMovingAverage(data: Array<number>, period: number): Array<number>
```

Arithmetic mean over a sliding window. Output length equals input length, with the first `period - 1` values set to `NaN`.

```javascript
const sma = simpleMovingAverage([10, 11, 12, 13, 14, 15], 3)
// [NaN, NaN, 11, 12, 13, 14]
```

---

### Exponential Moving Average (EMA)

```typescript
exponentialMovingAverage(data: Array<number>, period: number): Array<number>
```

Weighted average giving more importance to recent prices. Uses the standard SMA of the first `period` values as the seed, then applies the smoothing factor `k = 2 / (period + 1)`.

Output length = `input.length - period + 1`. First value corresponds to the candle at index `period - 1`.

```javascript
const ema = exponentialMovingAverage([10, 11, 12, 13, 14, 15, 16], 3)
// output starts at index 2 of input data
```

---

### Relative Strength Index (RSI)

```typescript
relativeStrengthIndex(prices: Array<number>, period: number): Array<number>
```

Momentum oscillator (0-100) using **Wilder's smoothing**:

- Initial average gain/loss = SMA of first `period` price changes
- Subsequent: `avg = (prev_avg * (period - 1) + current) / period`
- `RSI = 100 - 100 / (1 + RS)` where `RS = avg_gain / avg_loss`

Output length = `prices.length - period`. Requires at least `period + 1` data points.

```javascript
const rsi = relativeStrengthIndex(closePrices, 14)
```

---

### Bollinger Bands

```typescript
bollingerBands(
  data: Array<number>,
  period?: number,       // default: 20
  multiplier?: number    // default: 2.0
): BollingerBandsResult

interface BollingerBandsResult {
  upper: Array<number>
  middle: Array<number>  // SMA
  lower: Array<number>
}
```

Middle band = SMA. Upper/lower = SMA +/- `multiplier * population_std_dev`. Output length = `data.length - period + 1`.

```javascript
const bb = bollingerBands(closePrices, 20, 2.0)
// bb.upper, bb.middle, bb.lower
```

---

### Directional Movement Index (DMI / ADX)

```typescript
directionalMovementIndex(
  data: Array<MarketData>,
  period: number
): DmiResult

interface DmiResult {
  plusDi: Array<number>   // +DI values
  minusDi: Array<number>  // -DI values
  adx: Array<number>      // Average Directional Index
}
```

Measures trend direction and strength using Wilder's smoothing. Output arrays have length = input length. Values before `period` (+DI/-DI) or `period * 2` (ADX) are `NaN`. Requires at least `period * 2` data points.

```javascript
const dmi = directionalMovementIndex(marketData, 14)
// dmi.plusDi, dmi.minusDi, dmi.adx
```

---

### Stochastic Oscillator (%K)

```typescript
stochasticOscillator(
  data: Array<MarketData>,
  period: number
): Array<number>
```

`%K = 100 * (Close - Lowest_Low) / (Highest_High - Lowest_Low)`

Lookback window includes the current bar: `[i - period + 1, i]`. Returns 50.0 when the range is zero (flat prices). Output length = `data.length - period + 1`.

```javascript
const stoch = stochasticOscillator(marketData, 14)
```

---

### Stochastic Momentum Index (SMI)

```typescript
stochasticMomentumIndex(
  data: Array<MarketData>,
  lookbackPeriod?: number,    // q, default: 14
  firstSmoothing?: number,    // r, default: 3
  secondSmoothing?: number    // s, default: 3
): Array<number>
```

Blau's SMI formula: `SMI = 200 * EMA_s(EMA_r(D)) / EMA_s(EMA_r(R))`

Where `D = Close - Midpoint(HH, LL)` and `R = HH - LL`. Double EMA smoothing with separate periods for lookback (q), first smoothing (r), and second smoothing (s). Output range: [-100, 100]. First `lookback - 1` values are `NaN`.

```javascript
const smi = stochasticMomentumIndex(marketData, 14, 3, 3)
```

---

### Ichimoku Cloud

```typescript
ichimoku(
  data: Array<MarketData>,
  tenkanPeriod?: number,    // default: 9
  kijunPeriod?: number,     // default: 26
  senkouBPeriod?: number,   // default: 52
  chikouShift?: number      // default: 26
): Array<IchimokuData>

interface IchimokuData {
  tenkanSen: number     // Conversion Line
  kijunSen: number      // Base Line
  senkouSpanA: number   // Leading Span A (shifted forward)
  senkouSpanB: number   // Leading Span B (shifted forward)
  chikouSpan: number    // Lagging Span
}
```

Complete Ichimoku Kinko Hyo system. Output has one entry per input bar.

| Component | Formula | Displacement |
|-----------|---------|-------------|
| Tenkan-sen | (9-period HH + LL) / 2 | None |
| Kijun-sen | (26-period HH + LL) / 2 | None |
| Senkou Span A | (Tenkan + Kijun) / 2 | Forward `kijunPeriod` bars |
| Senkou Span B | (52-period HH + LL) / 2 | Forward `kijunPeriod` bars |
| Chikou Span | Close | Backward `chikouShift` bars |

Values are `NaN` when insufficient data is available for a given component.

```javascript
const ichi = ichimoku(marketData)
// ichi[i].tenkanSen, ichi[i].senkouSpanA, etc.
```

---

### Parabolic SAR

```typescript
parabolicSar(
  data: Array<MarketData>,
  start?: number,       // initial AF, default: 0.02
  increment?: number,   // AF increment, default: 0.02
  maxValue?: number     // max AF, default: 0.20
): Array<number>
```

Wilder's Parabolic Stop and Reverse. Tracks trend direction with an accelerating factor (AF) and extreme point (EP). SAR is clamped to the prior two bars' lows (uptrend) or highs (downtrend). On reversal, SAR flips to the previous EP and AF resets.

```javascript
const sar = parabolicSar(marketData, 0.02, 0.02, 0.2)
```

---

### Trends Meter

```typescript
trendsMeter(
  data: Array<MarketData>,
  period?: number    // default: 14
): Array<number>
```

Combines EMA-smoothed True Range with EMA-smoothed Momentum: `(EMA(TR) + EMA(Momentum)) / 2`. Output length equals input length, with first `period` values at 0.

```javascript
const tm = trendsMeter(marketData, 14)
```

---

### Pivot Points

```typescript
pivotPoints(data: Array<MarketData>): Array<number>
```

Standard (Floor) pivot points computed from the **previous bar's** HLC. Returns a flat array of 5 values per bar: `[PP, R1, R2, S1, S2, ...]`.

| Level | Formula |
|-------|---------|
| PP | (H + L + C) / 3 |
| R1 | 2 * PP - L |
| R2 | PP + (H - L) |
| S1 | 2 * PP - H |
| S2 | PP - (H - L) |

Output length = `(data.length - 1) * 5`. First bar has no pivots (no previous bar).

```javascript
const pivots = pivotPoints(marketData)
// pivots[0] = PP for bar 1, pivots[1] = R1 for bar 1, etc.
```

---

### K-Reversal

```typescript
kReversal(
  data: Array<MarketData>,
  period?: number,          // default: 14
  buyThreshold?: number,    // default: 20
  sellThreshold?: number    // default: 80
): KReversalResult

interface KReversalSignal {
  index: number
  price: number
  kValue: number
}

interface KReversalResult {
  kValues: Array<number>
  buySignals: Array<KReversalSignal>
  sellSignals: Array<KReversalSignal>
}
```

`K = 100 * (Close - Low_N) / (High_N - Low_N)`

Identifies potential reversals. K < `buyThreshold` suggests oversold (potential uptrend). K > `sellThreshold` suggests overbought (potential downtrend). First `period - 1` values are `NaN`.

```javascript
const kr = kReversal(marketData, 14, 20, 80)
// kr.kValues, kr.buySignals, kr.sellSignals
```

---

### Entry/Exit Signals

```typescript
entryExitSignals(
  data: Array<MarketData>,
  smaPeriod: number,
  emaPeriod: number,
  atrPeriod: number,
  threshold: number
): Array<Signal>

interface Signal {
  type: number   // 0 = Entry, 1 = Exit
  price: number
  index: number  // bar index in input data
}
```

Generates entry/exit signals by combining SMA, EMA, and ATR:
- **Entry** (type=0): price > SMA, price > EMA, price > SMA + ATR * threshold
- **Exit** (type=1): price < SMA, price < EMA, price < SMA - ATR * threshold

Signals alternate: no duplicate entries or exits in a row.

```javascript
const signals = entryExitSignals(marketData, 20, 12, 14, 1.5)
signals.forEach(s => console.log(s.type === 0 ? 'BUY' : 'SELL', s.price, s.index))
```

---

### Extract Important Levels

```typescript
extractImportantLevels(data: Array<number>): ImportantLevels

interface ImportantLevels {
  highestResistance: number
  lowestSupport: number
  averagePivot: number
  supports: Array<number>
  resistances: Array<number>
}
```

Detects local support and resistance levels using a 5-bar window peak/trough detection. `highestResistance` and `lowestSupport` include the global data extremes.

```javascript
const levels = extractImportantLevels(closePrices)
// levels.supports, levels.resistances, levels.averagePivot
```

---

## Charts

### Renko Chart

```typescript
renkoChart(
  prices: Array<number>,
  brickSize?: number    // default: 10
): Array<RenkoBrick>

interface RenkoBrick {
  price: number
  direction: string  // "up" or "down"
}
```

Creates Renko bricks based on a fixed price movement threshold. Multiple bricks can be generated from a single price movement.

```javascript
const renko = renkoChart(closePrices, 5.0)
// [{ price: 100, direction: "up" }, { price: 105, direction: "up" }, ...]
```

---

### Kagi Chart

```typescript
kagiChart(
  prices: Array<number>,
  reversalAmount?: number   // default: 20
): Array<KagiPoint>

interface KagiPoint {
  price: number
  direction: string  // "Yang" (uptrend high) or "Yin" (downtrend low)
}
```

Creates Kagi chart points. A new point is recorded when price reverses by at least `reversalAmount`. Yang marks local highs (before a downturn), Yin marks local lows (before an upturn).

```javascript
const kagi = kagiChart(closePrices, 10.0)
```

---

## NaN Handling

Many indicators produce `NaN` for early values where insufficient data is available (warmup period):

| Indicator | First valid index |
|-----------|------------------|
| SMA | `period - 1` |
| EMA | Output starts at `period - 1` (shorter array, no NaN) |
| RSI | Output starts at `period` (shorter array, no NaN) |
| Bollinger Bands | Output starts at `period - 1` (shorter array, no NaN) |
| Stochastic | Output starts at `period - 1` (shorter array, no NaN) |
| DMI +DI/-DI | `period` |
| DMI ADX | `period * 2 - 1` |
| Ichimoku | Varies per component |
| SMI | `lookback - 1` |
| K-Reversal | `period - 1` |

Always filter or check for `NaN` before using indicator values in calculations.

## Error Handling

All functions throw descriptive errors for invalid inputs:

```javascript
try {
  simpleMovingAverage([], 14)
} catch (e) {
  // "Data array cannot be empty"
}

try {
  simpleMovingAverage([1, 2, 3], 10)
} catch (e) {
  // "Data array length (3) is less than period (10)"
}
```

## Algorithm References

| Indicator | Reference |
|-----------|-----------|
| EMA | Standard SMA-seeded EMA with k = 2/(period+1) |
| RSI | Wilder's Relative Strength Index (1978) |
| ATR | Wilder's Average True Range with smoothing |
| DMI/ADX | Wilder's Directional Movement System |
| Parabolic SAR | Wilder's Parabolic Stop and Reverse |
| Bollinger Bands | John Bollinger (population std dev) |
| Ichimoku | Goichi Hosoda's Ichimoku Kinko Hyo |
| SMI | William Blau's Stochastic Momentum Index |
| Pivot Points | Standard Floor Pivot Points |
| Stochastic | George Lane's Stochastic Oscillator |
| K-Reversal | K-Reversal Momentum Indicator |

## Development

```bash
# Install dependencies
npm install

# Build (requires Rust toolchain)
npm run build

# Build debug
npm run build:debug

# Run tests
npm test
```

## License

MIT
