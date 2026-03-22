import { test } from '@japa/runner'
import pkg from '../../index.js'
const { kReversal } = pkg
import { generateTestData } from './lib.mjs'

test.group('KReversal', (group) => {

    test('valid input returns kValues of length equal to data length', ({ assert }) => {
        const data = generateTestData(50)
        const result = kReversal(data)
        // kValues always has the same length as the input data
        assert.lengthOf(result.kValues, 50)
    })

    test('result object has kValues, buySignals and sellSignals fields', ({ assert }) => {
        const data = generateTestData(50)
        const result = kReversal(data)
        assert.property(result, 'kValues')
        assert.property(result, 'buySignals')
        assert.property(result, 'sellSignals')
        assert.isArray(result.kValues)
        assert.isArray(result.buySignals)
        assert.isArray(result.sellSignals)
    })

    test('first (period - 1) kValues are NaN', ({ assert }) => {
        const data = generateTestData(50)
        const period = 14
        const result = kReversal(data, period)
        for (let i = 0; i < period - 1; i++) {
            assert.isTrue(isNaN(result.kValues[i]), `kValues[${i}] should be NaN`)
        }
    })

    test('kValues from index (period - 1) onwards are numbers between 0 and 100', ({ assert }) => {
        const data = generateTestData(50)
        const period = 14
        const result = kReversal(data, period)
        for (let i = period - 1; i < result.kValues.length; i++) {
            assert.isFalse(isNaN(result.kValues[i]), `kValues[${i}] should not be NaN`)
            assert.isAtLeast(result.kValues[i], 0)
            assert.isAtMost(result.kValues[i], 100)
        }
    })

    test('buySignals each have kValue below buyThreshold (default 20)', ({ assert }) => {
        const data = generateTestData(200)
        const result = kReversal(data, 14, 20, 80)
        result.buySignals.forEach((signal, i) => {
            assert.isBelow(signal.kValue, 20, `buySignals[${i}].kValue should be < 20`)
        })
    })

    test('sellSignals each have kValue above sellThreshold (default 80)', ({ assert }) => {
        const data = generateTestData(200)
        const result = kReversal(data, 14, 20, 80)
        result.sellSignals.forEach((signal, i) => {
            assert.isAbove(signal.kValue, 80, `sellSignals[${i}].kValue should be > 80`)
        })
    })

    test('each signal has index, price and kValue properties', ({ assert }) => {
        const data = generateTestData(200)
        const result = kReversal(data)
        const allSignals = [...result.buySignals, ...result.sellSignals]
        allSignals.forEach((signal) => {
            assert.property(signal, 'index')
            assert.property(signal, 'price')
            assert.property(signal, 'kValue')
            assert.isNumber(signal.index)
            assert.isNumber(signal.price)
            assert.isNumber(signal.kValue)
        })
    })

    test('signal indices are within valid range of input data', ({ assert }) => {
        const data = generateTestData(100)
        const period = 14
        const result = kReversal(data, period)
        const allSignals = [...result.buySignals, ...result.sellSignals]
        allSignals.forEach((signal) => {
            assert.isAtLeast(signal.index, period - 1)
            assert.isBelow(signal.index, data.length)
        })
    })

    test('custom buy/sell thresholds are respected', ({ assert }) => {
        const data = generateTestData(200)
        const buyThreshold = 30
        const sellThreshold = 70
        const result = kReversal(data, 14, buyThreshold, sellThreshold)
        result.buySignals.forEach((signal) => {
            assert.isBelow(signal.kValue, buyThreshold)
        })
        result.sellSignals.forEach((signal) => {
            assert.isAbove(signal.kValue, sellThreshold)
        })
    })

    test('flat prices produce kValue of 50 for all computed values', ({ assert }) => {
        // When high === low throughout, range is 0 and Rust returns 50.0
        const flatData = Array.from({ length: 20 }, (_, i) => ({
            date: new Date(2025, 0, i + 1).toISOString().split('T')[0],
            open: 100.0,
            high: 100.0,
            low: 100.0,
            close: 100.0,
            volume: 1000,
        }))
        const period = 5
        const result = kReversal(flatData, period)
        for (let i = period - 1; i < result.kValues.length; i++) {
            assert.approximately(result.kValues[i], 50, 0.001)
        }
    })

    test('empty data throws error', ({ assert }) => {
        try {
            kReversal([])
            assert.fail('Expected error was not thrown')
        } catch (error) {
            assert.equal(error.message, 'Not enough data for the given period')
        }
    })

    test('not enough data for given period throws error', ({ assert }) => {
        const data = generateTestData(5)
        try {
            kReversal(data, 14)
            assert.fail('Expected error was not thrown')
        } catch (error) {
            assert.equal(error.message, 'Not enough data for the given period')
        }
    })
})
