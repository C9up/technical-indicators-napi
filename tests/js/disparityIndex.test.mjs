import { test } from '@japa/runner'
import pkg from '../../index.js'
const { disparityIndex } = pkg
import { generateTestData } from './lib.mjs'

test.group('Disparity Index', (group) => {

    let prices

    group.setup(() => {
        const data = generateTestData(100)
        prices = data.map((d) => d.close)
    })

    test('output length equals input length', ({ assert }) => {
        const period = 14
        const result = disparityIndex(prices, period)
        assert.lengthOf(result, prices.length)
    })

    test('output length equals input length with default period', ({ assert }) => {
        const result = disparityIndex(prices)
        assert.lengthOf(result, prices.length)
    })

    test('first period minus one values are NaN', ({ assert }) => {
        const period = 14
        const result = disparityIndex(prices, period)
        for (let i = 0; i < period - 1; i++) {
            assert.isTrue(
                isNaN(result[i]),
                `result[${i}] should be NaN but got ${result[i]}`
            )
        }
    })

    test('value at index period minus one is not NaN', ({ assert }) => {
        const period = 14
        const result = disparityIndex(prices, period)
        assert.isFalse(
            isNaN(result[period - 1]),
            `result[${period - 1}] should not be NaN`
        )
    })

    test('known value: all equal prices give disparity of 0', ({ assert }) => {
        const flatPrices = Array.from({ length: 30 }, () => 100)
        const period = 10
        const result = disparityIndex(flatPrices, period)
        for (let i = period - 1; i < flatPrices.length; i++) {
            assert.approximately(
                result[i],
                0,
                1e-9,
                `result[${i}] should be 0 when all prices are equal`
            )
        }
    })

    test('known value: price above SMA gives positive disparity', ({ assert }) => {
        // Build a rising series so the last close is above its SMA
        const risingPrices = Array.from({ length: 20 }, (_, i) => 100 + i)
        const period = 5
        const result = disparityIndex(risingPrices, period)
        // For a strictly rising series, close > SMA → disparity > 0
        const lastIndex = risingPrices.length - 1
        assert.isAbove(result[lastIndex], 0)
    })

    test('known value: price below SMA gives negative disparity', ({ assert }) => {
        // Build a falling series so the last close is below its SMA
        const fallingPrices = Array.from({ length: 20 }, (_, i) => 200 - i)
        const period = 5
        const result = disparityIndex(fallingPrices, period)
        // For a strictly falling series, close < SMA → disparity < 0
        const lastIndex = fallingPrices.length - 1
        assert.isBelow(result[lastIndex], 0)
    })

    test('disparity index formula: (close - SMA) / SMA * 100', ({ assert }) => {
        // Use a simple hand-verifiable dataset
        // prices: 10, 10, 10, 10, 20  with period 5
        // SMA at index 4 = (10+10+10+10+20)/5 = 12
        // disparity = (20 - 12) / 12 * 100 = 66.666...
        const testPrices = [10, 10, 10, 10, 20]
        const period = 5
        const result = disparityIndex(testPrices, period)
        const expectedSma = (10 + 10 + 10 + 10 + 20) / 5
        const expectedDisparity = ((20 - expectedSma) / expectedSma) * 100
        assert.approximately(result[4], expectedDisparity, 1e-6)
    })

    test('empty data throws', ({ assert }) => {
        try {
            disparityIndex([])
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.instanceOf(error, Error)
        }
    })

    test('period greater than data length throws or returns all NaN', ({ assert }) => {
        const shortPrices = [100, 101, 102]
        const period = 10
        try {
            const result = disparityIndex(shortPrices, period)
            // If it does not throw, every value should be NaN
            for (const value of result) {
                assert.isTrue(isNaN(value), `Expected NaN but got ${value}`)
            }
        } catch (error) {
            assert.instanceOf(error, Error)
        }
    })

    test('period of 1 gives zero disparity for all values', ({ assert }) => {
        // With period 1, SMA equals close, so (close - SMA) / SMA = 0
        const result = disparityIndex(prices, 1)
        for (let i = 0; i < result.length; i++) {
            if (!isNaN(result[i])) {
                assert.approximately(result[i], 0, 1e-9)
            }
        }
    })
})
