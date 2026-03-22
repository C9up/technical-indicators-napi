import { test } from '@japa/runner'
import { entryExitSignals } from '../../index.js'
import { generateTestData } from './lib.mjs'

/**
 * entryExitSignals takes MarketData[] but the Rust signature accepts Vec<MarketData>
 * and internally extracts closes, highs, lows via process_market_data.
 * Signal: { type: 0 (entry) | 1 (exit), price: number, index: number }
 */

/** Build a simple ascending price dataset to maximize signal generation */
function buildTrendingData(count) {
    return Array.from({ length: count }, (_, i) => ({
        date: new Date(2025, 0, i + 1).toISOString().split('T')[0],
        open: 100 + i * 0.5,
        high: 102 + i * 0.5,
        low: 99 + i * 0.5,
        close: 100 + i * 0.5,
        volume: 1000,
    }))
}

test.group('EntryExitSignals', (group) => {

    test('returns an array', ({ assert }) => {
        const data = generateTestData(200)
        const result = entryExitSignals(data, 20, 10, 14, 1.5)
        assert.isArray(result)
    })

    test('each signal has type, price and index properties', ({ assert }) => {
        const data = generateTestData(200)
        const result = entryExitSignals(data, 20, 10, 14, 1.5)
        result.forEach((signal, i) => {
            assert.property(signal, 'type', `signal[${i}] missing type`)
            assert.property(signal, 'price', `signal[${i}] missing price`)
            assert.property(signal, 'index', `signal[${i}] missing index`)
        })
    })

    test('signal type is 0 (entry) or 1 (exit)', ({ assert }) => {
        const data = generateTestData(200)
        const result = entryExitSignals(data, 20, 10, 14, 1.5)
        result.forEach((signal, i) => {
            assert.oneOf(signal.type, [0, 1], `signal[${i}].type must be 0 or 1`)
        })
    })

    test('signal price is a positive finite number', ({ assert }) => {
        const data = generateTestData(200)
        const result = entryExitSignals(data, 20, 10, 14, 1.5)
        result.forEach((signal, i) => {
            assert.isNumber(signal.price, `signal[${i}].price is not a number`)
            assert.isAbove(signal.price, 0, `signal[${i}].price must be positive`)
            assert.isTrue(isFinite(signal.price), `signal[${i}].price must be finite`)
        })
    })

    test('signal index is within the bounds of the input data', ({ assert }) => {
        const data = generateTestData(200)
        const result = entryExitSignals(data, 20, 10, 14, 1.5)
        result.forEach((signal, i) => {
            assert.isAtLeast(signal.index, 0, `signal[${i}].index must be >= 0`)
            assert.isBelow(signal.index, data.length, `signal[${i}].index must be < data.length`)
        })
    })

    test('signals alternate: no two consecutive entries or exits', ({ assert }) => {
        const data = generateTestData(500)
        const result = entryExitSignals(data, 20, 10, 14, 1.5)
        for (let i = 1; i < result.length; i++) {
            assert.notEqual(
                result[i].type,
                result[i - 1].type,
                `signals[${i - 1}] and signals[${i}] must not have the same type`
            )
        }
    })

    test('signals are ordered by ascending index', ({ assert }) => {
        const data = generateTestData(200)
        const result = entryExitSignals(data, 20, 10, 14, 1.5)
        for (let i = 1; i < result.length; i++) {
            assert.isAbove(
                result[i].index,
                result[i - 1].index,
                `signal index must increase monotonically`
            )
        }
    })

    test('empty data returns empty array (no throw)', ({ assert }) => {
        const result = entryExitSignals([], 20, 10, 14, 1.5)
        assert.isArray(result)
        assert.lengthOf(result, 0)
    })

    test('insufficient data for any indicator returns empty array', ({ assert }) => {
        // Only 5 bars — far below any of smaPeriod(20), emaPeriod(10), atrPeriod(14)+1
        const data = generateTestData(5)
        const result = entryExitSignals(data, 20, 10, 14, 1.5)
        assert.isArray(result)
        assert.lengthOf(result, 0)
    })

    test('data just meeting minimum required length may produce signals or empty array', ({ assert }) => {
        // required_min_len = max(smaPeriod, emaPeriod, atrPeriod + 1) = max(10, 5, 6) = 10
        const data = generateTestData(10)
        const result = entryExitSignals(data, 10, 5, 5, 1.0)
        assert.isArray(result)
    })

    test('very large threshold suppresses all signals', ({ assert }) => {
        // threshold=1000 makes entry/exit conditions nearly impossible to trigger
        const data = generateTestData(500)
        const result = entryExitSignals(data, 20, 10, 14, 1000)
        assert.isArray(result)
        assert.lengthOf(result, 0)
    })
})
