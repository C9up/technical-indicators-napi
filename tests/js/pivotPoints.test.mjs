import { test } from '@japa/runner'
import { pivotPoints } from '../../index.js'
import { generateTestData } from './lib.mjs'

/**
 * pivotPoints(data: MarketData[]): number[]
 *
 * For each bar i (1..len), computes 5 levels from the PREVIOUS bar's HLC:
 *   PP  = (H + L + C) / 3
 *   R1  = 2 * PP - L
 *   R2  = PP + (H - L)
 *   S1  = 2 * PP - H
 *   S2  = PP - (H - L)
 *
 * Output layout per previous bar: [PP, R1, R2, S1, S2]
 * Output length = (data.length - 1) * 5
 */

test.group('PivotPoints', (group) => {

    test('output length equals (data.length - 1) * 5', ({ assert }) => {
        const data = generateTestData(20)
        const result = pivotPoints(data)
        assert.lengthOf(result, (20 - 1) * 5)
    })

    test('output length for 100 bars is 495', ({ assert }) => {
        const data = generateTestData(100)
        const result = pivotPoints(data)
        assert.lengthOf(result, 99 * 5)
    })

    test('known calculation: PP = (H + L + C) / 3 for first bar', ({ assert }) => {
        const data = [
            { date: '2025-01-01', open: 100, high: 110, low: 90, close: 105, volume: 1000 },
            { date: '2025-01-02', open: 105, high: 115, low: 95, close: 108, volume: 1100 },
        ]
        const result = pivotPoints(data)
        // First 5 values come from bar[0]: H=110, L=90, C=105
        const expectedPP = (110 + 90 + 105) / 3  // 101.666...
        assert.approximately(result[0], expectedPP, 0.001)
    })

    test('known calculation: R1 = 2*PP - L', ({ assert }) => {
        const data = [
            { date: '2025-01-01', open: 100, high: 110, low: 90, close: 105, volume: 1000 },
            { date: '2025-01-02', open: 105, high: 115, low: 95, close: 108, volume: 1100 },
        ]
        const result = pivotPoints(data)
        const pp = (110 + 90 + 105) / 3
        const expectedR1 = 2 * pp - 90
        assert.approximately(result[1], expectedR1, 0.001)
    })

    test('known calculation: R2 = PP + (H - L)', ({ assert }) => {
        const data = [
            { date: '2025-01-01', open: 100, high: 110, low: 90, close: 105, volume: 1000 },
            { date: '2025-01-02', open: 105, high: 115, low: 95, close: 108, volume: 1100 },
        ]
        const result = pivotPoints(data)
        const pp = (110 + 90 + 105) / 3
        const expectedR2 = pp + (110 - 90)
        assert.approximately(result[2], expectedR2, 0.001)
    })

    test('known calculation: S1 = 2*PP - H', ({ assert }) => {
        const data = [
            { date: '2025-01-01', open: 100, high: 110, low: 90, close: 105, volume: 1000 },
            { date: '2025-01-02', open: 105, high: 115, low: 95, close: 108, volume: 1100 },
        ]
        const result = pivotPoints(data)
        const pp = (110 + 90 + 105) / 3
        const expectedS1 = 2 * pp - 110
        assert.approximately(result[3], expectedS1, 0.001)
    })

    test('known calculation: S2 = PP - (H - L)', ({ assert }) => {
        const data = [
            { date: '2025-01-01', open: 100, high: 110, low: 90, close: 105, volume: 1000 },
            { date: '2025-01-02', open: 105, high: 115, low: 95, close: 108, volume: 1100 },
        ]
        const result = pivotPoints(data)
        const pp = (110 + 90 + 105) / 3
        const expectedS2 = pp - (110 - 90)
        assert.approximately(result[4], expectedS2, 0.001)
    })

    test('values are all finite numbers', ({ assert }) => {
        const data = generateTestData(30)
        const result = pivotPoints(data)
        result.forEach((value, i) => {
            assert.isTrue(isFinite(value), `result[${i}] must be finite`)
            assert.isFalse(isNaN(value), `result[${i}] must not be NaN`)
        })
    })

    test('R2 >= R1 >= PP >= S1 >= S2 for each group of 5 values', ({ assert }) => {
        const data = generateTestData(50)
        const result = pivotPoints(data)
        for (let i = 0; i < result.length; i += 5) {
            const [pp, r1, r2, s1, s2] = result.slice(i, i + 5)
            assert.isAtLeast(r2, r1, `R2 >= R1 at group ${i / 5}`)
            assert.isAtLeast(r1, pp, `R1 >= PP at group ${i / 5}`)
            assert.isAtLeast(pp, s1, `PP >= S1 at group ${i / 5}`)
            assert.isAtLeast(s1, s2, `S1 >= S2 at group ${i / 5}`)
        }
    })

    test('less than 2 data points throws error', ({ assert }) => {
        try {
            pivotPoints([{ date: '2025-01-01', open: 100, high: 110, low: 90, close: 105, volume: 1000 }])
            assert.fail('Expected error was not thrown')
        } catch (error) {
            assert.equal(error.message, 'Need at least 2 data points for pivot points')
        }
    })

    test('empty data throws error', ({ assert }) => {
        try {
            pivotPoints([])
            assert.fail('Expected error was not thrown')
        } catch (error) {
            assert.equal(error.message, 'Need at least 2 data points for pivot points')
        }
    })

    test('output layout: groups of 5 values per bar correspond to [PP, R1, R2, S1, S2]', ({ assert }) => {
        const data = [
            { date: '2025-01-01', open: 50, high: 60, low: 40, close: 55, volume: 500 },
            { date: '2025-01-02', open: 55, high: 65, low: 45, close: 58, volume: 600 },
            { date: '2025-01-03', open: 58, high: 68, low: 48, close: 62, volume: 700 },
        ]
        const result = pivotPoints(data)
        // Group 0: from bar[0] (H=60, L=40, C=55)
        const pp0 = (60 + 40 + 55) / 3
        assert.approximately(result[0], pp0, 0.001)
        assert.approximately(result[1], 2 * pp0 - 40, 0.001)
        assert.approximately(result[2], pp0 + (60 - 40), 0.001)
        assert.approximately(result[3], 2 * pp0 - 60, 0.001)
        assert.approximately(result[4], pp0 - (60 - 40), 0.001)
        // Group 1: from bar[1] (H=65, L=45, C=58)
        const pp1 = (65 + 45 + 58) / 3
        assert.approximately(result[5], pp1, 0.001)
        assert.approximately(result[6], 2 * pp1 - 45, 0.001)
    })
})
