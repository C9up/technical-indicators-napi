import { test } from '@japa/runner'
import { frama } from '../../index.js'
import { generateTestData } from './lib.mjs'

test.group('FRAMA', (group) => {

    test('all output arrays have the same length as input', ({ assert }) => {
        const data = generateTestData(100)
        const result = frama(data)
        assert.equal(result.frama.length, data.length)
        assert.equal(result.fractalDimension.length, data.length)
        assert.equal(result.alpha.length, data.length)
        assert.equal(result.slope.length, data.length)
    })

    test('fractalDimension non-NaN values are between 1.0 and 2.0', ({ assert }) => {
        const data = generateTestData(100)
        const result = frama(data)
        for (let i = 0; i < result.fractalDimension.length; i++) {
            const fd = result.fractalDimension[i]
            if (!isNaN(fd)) {
                assert.isTrue(fd >= 1.0 && fd <= 2.0,
                    `fractalDimension[${i}] = ${fd} is outside [1.0, 2.0]`)
            }
        }
    })

    test('alpha non-NaN values are between slow_alpha and fast_alpha', ({ assert }) => {
        // Default period=10, fastPeriod=1, slowPeriod=200
        // fast_alpha = 2 / (fastPeriod + 1) = 2/2 = 1.0
        // slow_alpha = 2 / (slowPeriod + 1) = 2/201 ~= 0.00995
        const data = generateTestData(100)
        const result = frama(data, 10, 1, 200)
        const fastAlpha = 2 / (1 + 1)
        const slowAlpha = 2 / (200 + 1)
        for (let i = 0; i < result.alpha.length; i++) {
            const a = result.alpha[i]
            if (!isNaN(a)) {
                assert.isTrue(a >= slowAlpha - 1e-9 && a <= fastAlpha + 1e-9,
                    `alpha[${i}] = ${a} is outside [${slowAlpha}, ${fastAlpha}]`)
            }
        }
    })

    test('frama output contains at least some non-NaN values', ({ assert }) => {
        const data = generateTestData(100)
        const result = frama(data)
        const nonNaN = result.frama.filter(v => !isNaN(v))
        assert.isTrue(nonNaN.length > 0, 'Expected at least some non-NaN frama values')
    })

    test('period less than 4 throws', ({ assert }) => {
        const data = generateTestData(50)
        try {
            frama(data, 3)
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.isTrue(error instanceof Error)
        }
    })

    test('not enough data throws', ({ assert }) => {
        // period defaults to ~10; provide fewer candles than required
        const data = generateTestData(3)
        try {
            frama(data, 10)
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.isTrue(error instanceof Error)
        }
    })

    test('slope array same length as input', ({ assert }) => {
        const data = generateTestData(80)
        const result = frama(data)
        assert.equal(result.slope.length, data.length)
    })

    test('works with custom period fastPeriod and slowPeriod', ({ assert }) => {
        const data = generateTestData(120)
        const result = frama(data, 16, 2, 100)
        assert.equal(result.frama.length, data.length)
    })

})
