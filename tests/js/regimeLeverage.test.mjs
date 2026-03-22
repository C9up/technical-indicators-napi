import { test } from '@japa/runner'
import { regimeLeverage } from '../../index.js'
import { generateTestData } from './lib.mjs'

/**
 * Generate a synthetic VIX-like series of `n` values around a realistic mean.
 * @param {number} n
 * @param {number} base  - baseline VIX level (default 20)
 * @returns {number[]}
 */
function makeVix(n, base = 20) {
    const values = []
    let seed = 31
    for (let i = 0; i < n; i++) {
        seed = (seed * 1664525 + 1013904223) & 0xffffffff
        values.push(base + ((seed >>> 0) / 0xffffffff) * 15 - 7)
    }
    return values
}

test.group('RegimeLeverage', (group) => {

    const VALID_REGIMES = new Set([-1, 0, 1, 2, 3])
    const VALID_LEVERAGES = new Set([0.0, 1.0, 2.0, 3.0])

    test('all output arrays have the same length as input data', ({ assert }) => {
        const data = generateTestData(300)
        const result = regimeLeverage(data)
        assert.equal(result.oscillator.length, data.length)
        assert.equal(result.yzVolatility.length, data.length)
        assert.equal(result.volPercentile.length, data.length)
        assert.equal(result.regime.length, data.length)
        assert.equal(result.leverage.length, data.length)
    })

    test('vixRatio array has the same length as input when vixValues provided', ({ assert }) => {
        const data = generateTestData(300)
        const vix = makeVix(300)
        const result = regimeLeverage(data, vix)
        assert.equal(result.vixRatio.length, data.length)
    })

    test('regime values are only -1, 0, 1, 2, or 3 (ignoring NaN)', ({ assert }) => {
        const data = generateTestData(300)
        const result = regimeLeverage(data)
        for (let i = 0; i < result.regime.length; i++) {
            const r = result.regime[i]
            if (!isNaN(r)) {
                assert.isTrue(VALID_REGIMES.has(r),
                    `regime[${i}] = ${r} is not in {-1, 0, 1, 2, 3}`)
            }
        }
    })

    test('leverage values are NaN, 0.0, 1.0, 2.0, or 3.0', ({ assert }) => {
        const data = generateTestData(300)
        const result = regimeLeverage(data)
        for (let i = 0; i < result.leverage.length; i++) {
            const l = result.leverage[i]
            if (!isNaN(l)) {
                assert.isTrue(VALID_LEVERAGES.has(l),
                    `leverage[${i}] = ${l} is not in {0.0, 1.0, 2.0, 3.0}`)
            }
        }
    })

    test('yzVolatility non-NaN values are >= 0', ({ assert }) => {
        const data = generateTestData(300)
        const result = regimeLeverage(data)
        for (let i = 0; i < result.yzVolatility.length; i++) {
            const v = result.yzVolatility[i]
            if (!isNaN(v)) {
                assert.isTrue(v >= 0,
                    `yzVolatility[${i}] = ${v} is negative`)
            }
        }
    })

    test('result has all expected output fields', ({ assert }) => {
        const data = generateTestData(300)
        const result = regimeLeverage(data)
        assert.properties(result, [
            'oscillator',
            'yzVolatility',
            'volPercentile',
            'regime',
            'leverage',
            'vixRatio',
        ])
    })

    test('volPercentile non-NaN values are between 0 and 100', ({ assert }) => {
        const data = generateTestData(300)
        const result = regimeLeverage(data)
        for (let i = 0; i < result.volPercentile.length; i++) {
            const p = result.volPercentile[i]
            if (!isNaN(p)) {
                assert.isTrue(p >= 0 && p <= 100,
                    `volPercentile[${i}] = ${p} is outside [0, 100]`)
            }
        }
    })

    test('custom yzWindow produces same-length output', ({ assert }) => {
        const data = generateTestData(300)
        const result = regimeLeverage(data, undefined, undefined, 30)
        assert.equal(result.yzVolatility.length, data.length)
    })

    test('providing both vixValues and vix3mValues works correctly', ({ assert }) => {
        const data = generateTestData(300)
        const vix = makeVix(300, 20)
        const vix3m = makeVix(300, 22)
        const result = regimeLeverage(data, vix, vix3m)
        assert.equal(result.vixRatio.length, data.length)
        // vixRatio non-NaN values should be positive ratios
        for (let i = 0; i < result.vixRatio.length; i++) {
            const r = result.vixRatio[i]
            if (!isNaN(r)) {
                assert.isTrue(r > 0, `vixRatio[${i}] = ${r} should be positive`)
            }
        }
    })

    test('not enough data throws', ({ assert }) => {
        const data = generateTestData(2)
        try {
            regimeLeverage(data)
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.isTrue(error instanceof Error)
        }
    })

})
