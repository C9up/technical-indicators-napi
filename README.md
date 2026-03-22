# @c9up/technical-indicators-napi

High-performance technical analysis and quantitative finance library written in Rust, compiled to native Node.js addon via [NAPI-RS](https://napi.rs). 35+ indicators, copulas, portfolio analytics, ML features, and chart types with zero JavaScript overhead.

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

Used by: `directionalMovementIndex`, `parabolicSar`, `stochasticOscillator`, `stochasticMomentumIndex`, `ichimoku`, `trendsMeter`, `pivotPoints`, `entryExitSignals`, `kReversal`, `awesomeOscillator`, `relativeVigorIndex`, `threeWayIndicator`, `choppinessIndex`, `candlestickPatterns`, `spreadEstimator`, `volatilityEngine`, `yangZhangVolatility`, `harVolatility`, `regimeLeverage`, `featureEngine`, `frama`, `patternMemory`

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

### Choppiness Index

```typescript
choppinessIndex(data: Array<MarketData>, period?: number, lowThreshold?: number, highThreshold?: number): ChoppinessResult

interface ChoppinessResult {
  chop: Array<number>     // CI values (0-100)
  signals: Array<number>  // 1=trending crossover, -1=choppy crossover, 0=neutral
}
```

`CI = 100 * log10(Sum(TR, N) / (HH - LL)) / log10(N)`. Low values (< 38.2) = trending, high values (> 61.8) = choppy/sideways.

---

### Disparity Index

```typescript
disparityIndex(prices: Array<number>, period?: number): Array<number>
```

`DI = 100 * (Close - SMA) / SMA`. Measures % distance from price to its moving average. Positive = above MA, negative = below.

---

### Awesome Oscillator (Bill Williams)

```typescript
awesomeOscillator(data: Array<MarketData>, fastPeriod?: number, slowPeriod?: number): AwesomeOscillatorResult

interface AwesomeOscillatorResult {
  ao: Array<number>        // SMA(fast, midpoints) - SMA(slow, midpoints)
  histogram: Array<number> // +1 rising, -1 falling, 0 neutral
}
```

---

### Relative Vigor Index (RVI)

```typescript
relativeVigorIndex(data: Array<MarketData>, period?: number): RviResult

interface RviResult {
  rvi: Array<number>    // RVI line (close-open vs high-low ratio, smoothed)
  signal: Array<number> // Signal line (4-bar symmetric weighted MA)
}
```

---

### Three Way Indicator

```typescript
threeWayIndicator(data: Array<MarketData>, fastSma?, slowSma?, rsiPeriod?, atrPeriod?, atrLookback?, signalThreshold?): ThreeWayResult

interface ThreeWayResult {
  score: Array<number>      // -3 to +3 composite
  trend: Array<number>      // SMA crossover: +1/-1
  momentum: Array<number>   // RSI vs 50: +1/-1
  volatility: Array<number> // ATR direction: +1/-1
  signals: Array<number>    // 1=strong buy, -1=strong sell
}
```

---

### Candlestick Patterns (13 patterns)

```typescript
candlestickPatterns(data: Array<MarketData>, bodyThreshold?: number): CandlestickPatterns

interface CandlestickPatterns {
  doji, engulfing, hammer, hangingMan, harami, morningStar, eveningStar,
  threeWhiteSoldiers, threeBlackCrows, shootingStar, invertedHammer,
  spinningTop, marubozu: Array<number>  // +1 bullish, -1 bearish, 0 none
  composite: Array<number>               // sum of all signals
}
```

Native Rust alternative to TA-Lib. Single-bar, two-bar, and three-bar patterns detected in one pass.

---

### FRAMA (Fractal Adaptive Moving Average)

```typescript
frama(data: Array<MarketData>, period?: number, fastPeriod?: number, slowPeriod?: number): FramaResult

interface FramaResult {
  frama: Array<number>            // Adaptive moving average
  fractalDimension: Array<number> // 1.0=trending, 2.0=choppy
  alpha: Array<number>            // Smoothing factor used
  slope: Array<number>            // Bar-to-bar FRAMA change
}
```

John Ehlers' FRAMA: EMA whose smoothing adapts via the fractal dimension of price data.

---

### Anchored Regression (Trend Detection)

```typescript
anchoredRegressionStatic(prices: Array<number>, anchorPeriod: number, bandMult?: number): AnchoredRegressionResult
anchoredRegressionRolling(prices: Array<number>, anchorPeriod: number, bandMult?: number): AnchoredRegressionResult

interface RegressionSegment { startIndex, endIndex, slope, intercept, stdDev, fitted, upperBand, lowerBand }
interface AnchoredRegressionResult {
  segments: Array<RegressionSegment>
  fitted, upperBand, lowerBand, slopes: Array<number>
}
```

Static: independent regression per fixed window. Rolling: regression updates bar-by-bar from each anchor reset.

---

## Volatility

### Yang-Zhang Volatility

```typescript
yangZhangVolatility(data: Array<MarketData>, window?: number): YangZhangResult

interface YangZhangResult {
  volatility: Array<number>    // Annualized YZ vol
  overnightVol, intradayVol, rogersSatchell: Array<number>
}
```

Combines overnight (close-to-open), intraday (open-to-close), and Rogers-Satchell components.

---

### Volatility Engine (Adaptive Stop-Loss)

```typescript
volatilityEngine(data: Array<MarketData>, atrPeriod?, volPeriod?, volHistoryLen?, volWarmup?,
  percentileLow?, percentileHigh?, lowVolMult?, medVolMult?, highVolMult?): VolatilityEngineResult

interface VolatilityEngineResult {
  atr, volatility, atrMultipliers, stopDistances, lowThresholds, highThresholds: Array<number>
  regimes: Array<number>  // 0=low, 1=medium, 2=high
}
```

ATR + rolling std dev of returns, classified into 3 regimes via percentiles. Each regime has a different ATR multiplier for dynamic stop-loss sizing.

---

### HAR-X Volatility Model

```typescript
harVolatility(data: Array<MarketData>, yzWindow?, harLookback?, percentileLow?, percentileHigh?, vixData?): HarResult

interface HarResult {
  predictedVol, volDaily, volWeekly, volMonthly: Array<number>
  regime: Array<number>   // 0=low, 1=medium, 2=high
  exposure: Array<number> // 2.0 (low vol), 1.0 (medium), 0.0 (high)
}
```

Heterogeneous Autoregressive model combining daily/weekly/monthly Yang-Zhang volatility via rolling OLS. Optional VIX integration.

---

### Regime Leverage (MRALS)

```typescript
regimeLeverage(data: Array<MarketData>, vixValues?, vix3mValues?, yzWindow?, emaFast?, emaSlow?,
  oscillatorSmooth?, volLookback?, trendPeriod?): RegimeLeverageResult

interface RegimeLeverageResult {
  oscillator, yzVolatility, volPercentile, vixRatio: Array<number>
  regime: Array<number>   // 0=Defensive, 1=Moderate, 2=Bullish, 3=Aggressive
  leverage: Array<number> // 0.0, 1.0, 2.0, or 3.0
}
```

4-regime classification with hybrid oscillator (momentum + relative strength + VIX). Optional VIX/VIX3M integration.

---

## Probability & Statistics

### Conditional Probability

```typescript
conditionalProbability(prices, firstMoveDays, secondMoveDays, firstThreshold, secondThreshold): ConditionalProbabilityResult
conditionalProbabilityMatrix(prices, firstMoveDays, secondMoveDays, firstThresholds[], secondThresholds[]): ConditionalMatrixEntry[]
```

P(second move >= Y% in M days | first move >= X% in N days). Matrix version for heatmaps.

---

### Spread Estimators

```typescript
spreadEstimator(data: Array<MarketData>, window: number): { spreads, signedSpreads: Array<number> }
rollSpreadEstimator(prices: Array<number>, window: number): Array<number>
corwinSchultzSpreadEstimator(data: Array<MarketData>, window: number): Array<number>
```

Three bid-ask spread estimation methods: Ardia et al. (2024) OHLC GMM, Roll (1984), Corwin-Schultz (2012).

---

### Pattern Memory (Lorentzian k-NN)

```typescript
patternMemory(data: Array<MarketData>, kNeighbors?, lookback?, window?, forwardBars?): PatternMemoryResult

interface PatternMemoryResult {
  signal, normalizedSignal, avgDistance: Array<number>
  bullishCount, bearishCount: Array<number>
}
```

Non-parametric directional signal. Encodes market state as 5-indicator feature vector, finds k-nearest past patterns via Lorentzian distance, sums their forward labels.

---

### Gaussian Mixture Model

```typescript
gaussianMixture(data: Array<number>, nFeatures: number, nComponents?, maxIterations?, tolerance?, normalize?, seed?): GmmResult

interface GmmResult {
  labels: Array<number>
  probabilities: Array<number>  // flat, nPoints * nComponents
  clusters: Array<{ id, mean, variance, weight, count }>
  bic: number
  logLikelihood: number
}
```

EM-based clustering for market regime detection. K-means++ initialization, BIC for model selection.

---

### Options Flow Scoring

```typescript
optionsFlowScore(contracts: Array<OptionContract>, spotPrice: number, topN?, kOtm?, minVolume?, minOi?,
  capOiVol?, wOi?, wOv?, wOtm?): Array<ScoredOption>
```

Composite scoring for institutional activity detection. Ranks contracts by OI z-score, OI/Volume stickiness, and OTM distance.

---

## Portfolio Analytics

### Performance Metrics

```typescript
performanceMetrics(returns: Array<number>, riskFreeRate?, periodsPerYear?): PerformanceMetrics
sharpeRatio(returns, riskFreeRate?, periodsPerYear?): number
sortinoRatio(returns, riskFreeRate?, periodsPerYear?): number
maxDrawdown(returns): number

interface PerformanceMetrics {
  sharpeRatio, sortinoRatio, calmarRatio, maxDrawdown, maxDrawdownDuration,
  totalReturn, annualizedReturn, annualizedVolatility, winRate, profitFactor,
  payoffRatio, skewness, kurtosis, var95, cvar95: number
}
```

---

### Markowitz Portfolio Analysis

```typescript
covarianceMatrix(returnsFlat: Array<number>, nAssets: number): CovarianceResult
portfolioStats(returnsFlat, nAssets, weights, riskFreeRate?): PortfolioStats
efficientFrontier(returnsFlat, nAssets, nPoints?, riskFreeRate?): EfficientFrontierResult
```

Covariance/correlation matrices, portfolio return/risk for given weights, and the full Markowitz efficient frontier with GMVP and max Sharpe portfolio.

---

### ML Feature Engine

```typescript
featureEngine(data: Array<MarketData>): Array<FeatureRow>
```

Generates ~35 features per bar: returns (1/5/10/20), volatility (TR, ATR, std dev), momentum (RSI, ROC), moving averages (SMA, EMA), MACD (line/signal/histogram), Bollinger (%B, bandwidth), price position, volume ratios, candle features, trend signals. Ready for scikit-learn / XGBoost / TensorFlow.

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
| Choppiness Index | `period` |
| Disparity Index | `period - 1` |
| Awesome Oscillator | `slowPeriod - 1` |
| RVI | `period + 5` |
| FRAMA | `period` |
| Yang-Zhang | `window` |
| HAR Volatility | `yzWindow + 22` |
| Volatility Engine | `volWarmup + volPeriod` |
| Feature Engine | 50 (fixed warmup) |

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

## Copulas (Risk Management)

Statistical tools for modelling dependence between assets beyond linear correlation. Used for portfolio risk management and scenario analysis.

### Quantile Transform

```typescript
quantileTransform(data: Array<number>): Array<number>
```

Converts data to uniform [0,1] distribution using empirical CDF (rank-based). Required preprocessing step before fitting copulas.

---

### Copula Sampling

```typescript
gaussianCopulaSample(rho: number, nSamples: number, seed?: number): CopulaSample
claytonCopulaSample(theta: number, nSamples: number, seed?: number): CopulaSample
gumbelCopulaSample(theta: number, nSamples: number, seed?: number): CopulaSample
frankCopulaSample(theta: number, nSamples: number, seed?: number): CopulaSample

interface CopulaSample {
  u: Array<number>  // first variable [0,1]
  v: Array<number>  // second variable [0,1]
}
```

Generate correlated samples from bivariate copulas. Optional seed for reproducibility.

| Copula | Parameter | Range | Tail Dependence |
|--------|-----------|-------|-----------------|
| Gaussian | rho (correlation) | [-1, 1] | None |
| Clayton | theta | (0, +inf) | Lower tail |
| Gumbel | theta | [1, +inf) | Upper tail |
| Frank | theta | (-inf, +inf) \ {0} | None (symmetric) |

```javascript
import { gaussianCopulaSample, claytonCopulaSample } from '@c9up/technical-indicators-napi'

// Gaussian: correlated samples with rho=0.7
const gauss = gaussianCopulaSample(0.7, 1000, 42)

// Clayton: strong lower tail dependence
const clay = claytonCopulaSample(2.0, 1000)
```

---

### Conditional Sampling

```typescript
gaussianConditionalSample(
  u1: number,        // conditioning value in [0,1]
  rho: number,       // correlation parameter
  nSamples: number,
  seed?: number
): CopulaSample
```

Sample the second variable given a fixed value for the first. Core building block for scenario analysis.

---

### Copula Fitting

```typescript
fitCopula(
  u: Array<number>,
  v: Array<number>,
  copulaType: string   // "gaussian" | "clayton" | "gumbel" | "frank"
): CopulaFitResult

interface CopulaFitResult {
  copulaType: string
  parameter: number      // fitted rho or theta
  logLikelihood: number  // goodness of fit
}
```

Fit a copula to uniform-transformed data via maximum likelihood (Gaussian) or Kendall's tau inversion (Archimedean copulas).

```javascript
import { quantileTransform, fitCopula } from '@c9up/technical-indicators-napi'

const u = quantileTransform(stockAReturns)
const v = quantileTransform(stockBReturns)
const fit = fitCopula(u, v, 'gaussian')
console.log(`Correlation: ${fit.parameter}, LL: ${fit.logLikelihood}`)
```

---

### Portfolio Scenario Simulation

```typescript
portfolioScenario(
  returnsData: Array<Array<number>>,  // [marketReturns, asset1Returns, asset2Returns, ...]
  marketDrop: number,                  // e.g. -0.05 for 5% drop
  copulaType?: string,                 // default: "gaussian"
  nSimulations?: number                // default: 1000
): Array<ScenarioResult>

interface ScenarioResult {
  ticker: string
  meanReturn: number
  worstCase: number        // 5th percentile
  bestCase: number         // 95th percentile
  simulatedReturns: Array<number>
}
```

Simulates how portfolio assets would behave given a market shock. Fits a copula between the market and each asset, then uses conditional sampling to generate scenarios.

```javascript
import { portfolioScenario } from '@c9up/technical-indicators-napi'

// First array = market returns, subsequent = individual assets
const results = portfolioScenario(
  [spxReturns, msftReturns, aaplReturns, googlReturns],
  -0.05,       // 5% market drop
  'gaussian',
  5000
)

results.forEach(r => {
  console.log(`${r.ticker}: mean=${r.meanReturn}, worst=${r.worstCase}, best=${r.bestCase}`)
})
```

---

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
| Gaussian Copula | Multivariate normal dependence structure |
| Clayton Copula | Archimedean copula (lower tail dependence) |
| Gumbel Copula | Archimedean copula (upper tail dependence) |
| Frank Copula | Archimedean copula (symmetric, no tail dependence) |
| Choppiness Index | Dreiss (1993) |
| Awesome Oscillator | Bill Williams |
| RVI | John Ehlers' Relative Vigor Index |
| FRAMA | John Ehlers' Fractal Adaptive Moving Average |
| Yang-Zhang | Yang & Zhang (2000) volatility estimator |
| HAR | Corsi (2009) Heterogeneous Autoregressive model |
| Ardia Spread | Ardia, Guidotti & Kroencke (2024) |
| Roll Spread | Roll (1984) serial covariance |
| Corwin-Schultz Spread | Corwin & Schultz (2012) high-low |
| Pattern Memory | Lorentzian k-NN classification |
| GMM | Expectation-Maximization with diagonal covariance |

## Development

```bash
# Install dependencies
npm install

# Build (requires Rust toolchain)
npm run build

# Build debug
npm run build:debug

# Run tests (290 tests)
npm test
```

## License

MIT
