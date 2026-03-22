import { test } from '@japa/runner'
import pkg from '../../index.js'
const { candlestickPatterns } = pkg
import { generateTestData } from './lib.mjs'

/**
 * All pattern arrays in the result except `composite`.
 * Keep in sync with the function's documented output fields.
 */
const PATTERN_KEYS = [
    'doji',
    'engulfing',
    'hammer',
    'hangingMan',
    'harami',
    'morningStar',
    'eveningStar',
    'threeWhiteSoldiers',
    'threeBlackCrows',
    'shootingStar',
    'invertedHammer',
    'spinningTop',
    'marubozu',
]

/**
 * Keys that are actually summed into the composite value.
 * spinningTop is excluded from the composite calculation in Rust.
 */
const COMPOSITE_KEYS = [
    'doji',
    'engulfing',
    'hammer',
    'hangingMan',
    'harami',
    'morningStar',
    'eveningStar',
    'threeWhiteSoldiers',
    'threeBlackCrows',
    'shootingStar',
    'invertedHammer',
    'marubozu',
]

test.group('CandlestickPatterns', (group) => {

    test('all pattern arrays have the same length as the input', ({ assert }) => {
        const data = generateTestData(50)
        const result = candlestickPatterns(data)
        for (const key of PATTERN_KEYS) {
            assert.equal(result[key].length, data.length,
                `Pattern "${key}" length ${result[key].length} does not match input length ${data.length}`)
        }
        assert.equal(result.composite.length, data.length)
    })

    test('all individual pattern values are -1, 0, or 1', ({ assert }) => {
        const data = generateTestData(50)
        const result = candlestickPatterns(data)
        const valid = new Set([-1, 0, 1])
        for (const key of PATTERN_KEYS) {
            for (let i = 0; i < result[key].length; i++) {
                assert.isTrue(valid.has(result[key][i]),
                    `Pattern "${key}" at index ${i} has invalid value ${result[key][i]}`)
            }
        }
    })

    test('composite[i] equals sum of all pattern values at index i', ({ assert }) => {
        const data = generateTestData(50)
        const result = candlestickPatterns(data)
        for (let i = 0; i < data.length; i++) {
            const expected = COMPOSITE_KEYS.reduce((sum, key) => sum + result[key][i], 0)
            assert.equal(result.composite[i], expected,
                `composite[${i}] expected ${expected}, got ${result.composite[i]}`)
        }
    })

    test('result contains the composite field', ({ assert }) => {
        const data = generateTestData(20)
        const result = candlestickPatterns(data)
        assert.property(result, 'composite')
    })

    test('works with default bodyThreshold', ({ assert }) => {
        const data = generateTestData(30)
        // Should not throw
        const result = candlestickPatterns(data)
        assert.equal(result.composite.length, data.length)
    })

    test('custom bodyThreshold still produces valid results', ({ assert }) => {
        const data = generateTestData(30)
        const result = candlestickPatterns(data, 0.1)
        const valid = new Set([-1, 0, 1])
        for (const key of PATTERN_KEYS) {
            for (const val of result[key]) {
                assert.isTrue(valid.has(val), `Value ${val} is not -1, 0, or 1`)
            }
        }
    })

    test('less than 3 data points throws', ({ assert }) => {
        const data = generateTestData(2)
        try {
            candlestickPatterns(data)
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.isTrue(error instanceof Error)
        }
    })

    test('exactly 3 data points does not throw', ({ assert }) => {
        const data = generateTestData(3)
        // Should not throw — 3 is the minimum for some patterns
        const result = candlestickPatterns(data)
        assert.equal(result.composite.length, data.length)
    })

})
