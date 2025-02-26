import { test } from '@japa/runner'
import { stochasticMomentumIndex } from '../../index.js'

test.group('StochasticMomentumIndex', async (group) => {

    let prices

    group.setup(async () => {
        prices = [
            { high: 2,   low: 1,   close: 1.5, open: 1.2, volume: 1000, date: "2025-01-01" },
            { high: 2.5, low: 1.2, close: 2,   open: 1.5, volume: 1100, date: "2025-01-02" },
            { high: 3,   low: 1.4, close: 2.5, open: 2,   volume: 1200, date: "2025-01-03" },
            { high: 2.8, low: 1.3, close: 2.2, open: 2.5, volume: 1150, date: "2025-01-04" },
            { high: 3.2, low: 1.5, close: 2.8, open: 2.2, volume: 1300, date: "2025-01-05" },
            { high: 4.8, low: 2.3, close: 3.2, open: 2.8, volume: 1150, date: "2025-01-06" },
            { high: 3.2, low: 1.5, close: 3.8, open: 3.2, volume: 1300, date: "2025-01-07" },
        ]
    })

    test('should return NaN for indices with insufficient data', async ({ assert }) => {
        // Define the period parameters
        const period_k = 14
        const period_d = 3

        // Calculate the SMI values
        const result = stochasticMomentumIndex(prices, period_k, period_d)
        // The first two values should be NaN because there isn't enough data for computation
        assert.isTrue(isNaN(result[0]))
        assert.isTrue(isNaN(result[1]))
    })

    test('should compute correct SMI for index 2', async ({ assert }) => {
        const period_l = 2
        const period_h = 2
        const result = stochasticMomentumIndex(prices, period_l, period_h)
        //console.log(result)
        assert.approximately(result[1], 16.5, 0.2);
    })

    test('should compute correct SMI for index 3', async ({ assert }) => {
        // Define the period parameters
        const period_l = 3
        const period_h = 3
        const result = stochasticMomentumIndex(prices, period_l, period_h)
        assert.approximately(result[2], 25, 0.1);
    })

    test('should compute correct SMI for index 4', async ({ assert }) => {
        // Define the period parameters
        const period_l = 4
        const period_h = 4
        const result = stochasticMomentumIndex(prices, period_l, period_h)
        assert.approximately(result[3], 10, .1)
    })
})
