/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export interface RenkoBrick {
  price: number
  direction: string
}
export declare function renkoChart(prices: Array<number>, brickSize: number): Array<RenkoBrick>
export interface KagiPoint {
  price: number
  direction: string
}
export declare function kagiChart(prices: Array<number>, reversalAmount: number): Array<KagiPoint>
export declare function lowHighOpenCloseVolumeDateToArray(data: Array<MarketData>): MarketDataResult
export interface BollingerBandsResult {
  middle: Array<number>
  upper: Array<number>
  lower: Array<number>
}
export interface MarketData {
  low: number
  high: number
  open: number
  close: number
  volume: number
  date: string
}
export interface MarketDataResult {
  lows: Array<number>
  highs: Array<number>
  opens: Array<number>
  closes: Array<number>
  volumes: Array<number>
  dates: Array<string>
}
export interface BollingerBandsResult {
  upper: Array<number>
  middle: Array<number>
  lower: Array<number>
}
export declare function bollingerBands(data: Array<number>, period?: number | undefined | null, multiplier?: number | undefined | null): BollingerBandsResult
export interface DmiResult {
  plusDi: Array<number>
  minusDi: Array<number>
  adx: Array<number>
}
export declare function directionalMovementIndex(data: Array<MarketData>, period: number): DmiResult
export interface Signal {
  type: number
  price: number
  index: number
}
export declare function entryExitSignals(data: Array<number>, smaPeriod: number, emaPeriod: number, atrPeriod: number, threshold: number): Array<Signal>
export declare function exponentialMovingAverage(data: Array<number>, period: number): Array<number>
export interface ImportantLevels {
  highestResistance: number
  lowestSupport: number
  averagePivot: number
  supports: Array<number>
  resistances: Array<number>
}
export declare function extractImportantLevels(data: Array<number>): ImportantLevels
export interface IchimokuData {
  tenkanSen?: number
  kijunSen?: number
  senkouSpanA?: number
  senkouSpanB?: number
  chikouSpan?: number
}
export declare function ichimoku(data: Array<MarketData>, tenkanPeriod?: number, kijunPeriod?: number, senkouBPeriod?: number, chikouShift?: number): Array<IchimokuData>
export declare function parabolicSar(data: Array<MarketData>, start?: number | undefined | null, increment?: number | undefined | null, maxValue?: number | undefined | null): Array<number>
export declare function pivotPoints(data: Array<MarketData>): Array<number>
export declare function relativeStrengthIndex(prices: Array<number>, period: number): Array<number>
export declare function simpleMovingAverage(data: Array<number>, period: number): Array<number>
export declare function stochasticMomentumIndex(data: Array<MarketData>, periodK?: number | undefined | null, periodD?: number | undefined | null): Array<number>
export declare function stochasticOscillator(data: Array<MarketData>, period: number): Array<number>
export declare function trendsMeter(data: Array<MarketData>, period?: number | undefined | null): Array<number>
export declare function sum(a: number, b: number): number
