import { test } from '@japa/runner'
import pkg from '../../index.js'
const { stochasticMomentumIndex } = pkg
import { generateTestData } from './lib.mjs'

test.group('StochasticMomentumIndex', async (group) => {

    let smallData
    let largeData

    group.setup(async () => {
        smallData = [
            { high: 2,   low: 1,   close: 1.5, open: 1.2, volume: 1000, date: "2025-01-01" },
            { high: 2.5, low: 1.2, close: 2,   open: 1.5, volume: 1100, date: "2025-01-02" },
            { high: 3,   low: 1.4, close: 2.5, open: 2,   volume: 1200, date: "2025-01-03" },
            { high: 2.8, low: 1.3, close: 2.2, open: 2.5, volume: 1150, date: "2025-01-04" },
            { high: 3.2, low: 1.5, close: 2.8, open: 2.2, volume: 1300, date: "2025-01-05" },
            { high: 4.8, low: 2.3, close: 3.2, open: 2.8, volume: 1150, date: "2025-01-06" },
            { high: 3.2, low: 1.5, close: 3.8, open: 3.2, volume: 1300, date: "2025-01-07" },
        ]
        largeData = generateTestData(100)
    })

    test('should return NaN for indices with insufficient data', async ({ assert }) => {
        const result = stochasticMomentumIndex(largeData, 14, 3, 3)
        // First lookback-1 values should be NaN
        for (let i = 0; i < 13; i++) {
            assert.isTrue(isNaN(result[i]), `result[${i}] should be NaN during warmup`)
        }
    })

    test('valid input returns array with NaN padding and values in [-100, 100]', async ({ assert }) => {
        const result = stochasticMomentumIndex(largeData, 14, 3, 3)

        // Should have some results
        assert.isTrue(result.length > 0)

        // Non-NaN values should be within [-100, 100]
        for (let i = 0; i < result.length; i++) {
            if (!isNaN(result[i])) {
                assert.isTrue(result[i] >= -100 && result[i] <= 100,
                    `result[${i}] = ${result[i]} is outside [-100, 100]`)
            }
        }
    })

    test('uses default parameters when none provided', async ({ assert }) => {
        const result = stochasticMomentumIndex(largeData)
        assert.isTrue(result.length > 0)
    })

    test('custom lookback and smoothing periods work', async ({ assert }) => {
        const result = stochasticMomentumIndex(largeData, 10, 5, 5)
        assert.isTrue(result.length > 0)

        for (let i = 0; i < result.length; i++) {
            if (!isNaN(result[i])) {
                assert.isTrue(result[i] >= -100 && result[i] <= 100,
                    `result[${i}] = ${result[i]} is outside [-100, 100]`)
            }
        }
    })
})
