import { test } from '@japa/runner'
import { stochasticOscillator } from '../../index.js'
import { generateTestData } from './lib.mjs'

test.group('StochasticOscillator', (group) => {

    test('valid input with 100 bars and period 14 returns array of correct length', ({ assert }) => {
        const data = generateTestData(100)
        const period = 14
        const result = stochasticOscillator(data, period)
        // Output length = closes.length - period + 1
        assert.lengthOf(result, 100 - period + 1)
    })

    test('all values are between 0 and 100 for random market data', ({ assert }) => {
        const data = generateTestData(100)
        const result = stochasticOscillator(data, 14)
        result.forEach((value) => {
            assert.isAtLeast(value, 0)
            assert.isAtMost(value, 100)
        })
    })

    test('flat prices (all equal high/low/close) return 50 for all values', ({ assert }) => {
        // When highest_high === lowest_low, range is 0 and the Rust impl returns 50.0
        const flatData = Array.from({ length: 20 }, (_, i) => ({
            date: new Date(2025, 0, i + 1).toISOString().split('T')[0],
            open: 100.0,
            high: 100.0,
            low: 100.0,
            close: 100.0,
            volume: 1000,
        }))
        const result = stochasticOscillator(flatData, 14)
        result.forEach((value) => {
            assert.approximately(value, 50, 0.001)
        })
    })

    test('close at period high returns 100', ({ assert }) => {
        // Close equals the period high => %K = 100
        const data = Array.from({ length: 15 }, (_, i) => ({
            date: new Date(2025, 0, i + 1).toISOString().split('T')[0],
            open: 100.0,
            high: 110.0,
            low: 90.0,
            close: 110.0,
            volume: 1000,
        }))
        const result = stochasticOscillator(data, 14)
        assert.approximately(result[0], 100, 0.001)
    })

    test('close at period low returns 0', ({ assert }) => {
        // Close equals the period low => %K = 0
        const data = Array.from({ length: 15 }, (_, i) => ({
            date: new Date(2025, 0, i + 1).toISOString().split('T')[0],
            open: 100.0,
            high: 110.0,
            low: 90.0,
            close: 90.0,
            volume: 1000,
        }))
        const result = stochasticOscillator(data, 14)
        assert.approximately(result[0], 0, 0.001)
    })

    test('period 0 throws error', ({ assert }) => {
        const data = generateTestData(20)
        try {
            stochasticOscillator(data, 0)
            assert.fail('Expected error was not thrown')
        } catch (error) {
            assert.equal(error.message, 'Period must be greater than 0')
        }
    })

    test('not enough data for the given period throws error', ({ assert }) => {
        const data = generateTestData(5)
        try {
            stochasticOscillator(data, 14)
            assert.fail('Expected error was not thrown')
        } catch (error) {
            assert.equal(error.message, 'Not enough data for the given period')
        }
    })

    test('empty data throws error', ({ assert }) => {
        try {
            stochasticOscillator([], 14)
            assert.fail('Expected error was not thrown')
        } catch (error) {
            assert.equal(error.message, 'Not enough data for the given period')
        }
    })

    test('known calculation: close midpoint of range returns ~50', ({ assert }) => {
        // With close exactly at midpoint of [low, high], %K = 50
        const data = Array.from({ length: 15 }, (_, i) => ({
            date: new Date(2025, 0, i + 1).toISOString().split('T')[0],
            open: 100.0,
            high: 110.0,
            low: 90.0,
            close: 100.0,
            volume: 1000,
        }))
        const result = stochasticOscillator(data, 14)
        assert.approximately(result[0], 50, 0.001)
    })
})
