import { test } from '@japa/runner'
import pkg from '../../index.js'
const { relativeStrengthIndex } = pkg

test.group('RelativeStrengthIndex', (group) => {

    test('valid input returns array of correct length (prices.length - period)', ({ assert }) => {
        const prices = Array.from({ length: 20 }, (_, i) => 100 + i)
        const period = 14
        const result = relativeStrengthIndex(prices, period)
        // Rust impl: output length = changes.len() - period + 1 = (prices.length - 1) - period + 1
        assert.lengthOf(result, prices.length - period)
    })

    test('all returned values are between 0 and 100', ({ assert }) => {
        const prices = Array.from({ length: 30 }, (_, i) => 50 + Math.sin(i * 0.5) * 10)
        const result = relativeStrengthIndex(prices, 14)
        result.forEach((value) => {
            assert.isAtLeast(value, 0)
            assert.isAtMost(value, 100)
        })
    })

    test('flat prices (all gains = 0, all losses = 0) returns 100 for all values', ({ assert }) => {
        // When all prices are equal, avg_gain = 0 and avg_loss = 0, so RSI = 100
        const prices = Array.from({ length: 20 }, () => 50.0)
        const result = relativeStrengthIndex(prices, 14)
        result.forEach((value) => {
            assert.approximately(value, 100, 0.001)
        })
    })

    test('strictly increasing prices return RSI of 100 (all gains, no losses)', ({ assert }) => {
        const prices = Array.from({ length: 20 }, (_, i) => 100 + i)
        const result = relativeStrengthIndex(prices, 14)
        result.forEach((value) => {
            assert.approximately(value, 100, 0.001)
        })
    })

    test('strictly decreasing prices return RSI of 0 (no gains, all losses)', ({ assert }) => {
        const prices = Array.from({ length: 20 }, (_, i) => 200 - i)
        const result = relativeStrengthIndex(prices, 14)
        result.forEach((value) => {
            assert.approximately(value, 0, 0.001)
        })
    })

    test('period 0 throws error', ({ assert }) => {
        const prices = [1.0, 2.0, 3.0, 4.0, 5.0]
        try {
            relativeStrengthIndex(prices, 0)
            assert.fail('Expected error was not thrown')
        } catch (error) {
            assert.equal(error.message, 'Period must be greater than 0')
        }
    })

    test('empty array throws error', ({ assert }) => {
        try {
            relativeStrengthIndex([], 14)
            assert.fail('Expected error was not thrown')
        } catch (error) {
            assert.isTrue(error.message.includes('Not enough data'))
        }
    })

    test('not enough data for given period throws error', ({ assert }) => {
        // Need at least period + 1 prices
        const prices = [100.0, 101.0, 102.0]
        try {
            relativeStrengthIndex(prices, 14)
            assert.fail('Expected error was not thrown')
        } catch (error) {
            assert.isTrue(error.message.includes('Not enough data'))
        }
    })

    test('known values: alternating up/down produces RSI near 50', ({ assert }) => {
        // Alternating +1/-1 changes produce equal avg_gain and avg_loss => RSI = 50
        const prices = []
        let price = 100
        for (let i = 0; i < 20; i++) {
            prices.push(price)
            price += i % 2 === 0 ? 1 : -1
        }
        const result = relativeStrengthIndex(prices, 14)
        // After Wilder smoothing stabilizes the first value should be close to 50
        assert.approximately(result[0], 50, 1)
    })
})
