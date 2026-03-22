import { test } from '@japa/runner'
import pkg from '../../index.js'
const { exponentialMovingAverage } = pkg

test.group('ExponentialMovingAverage', () => {
    test('test invalid period zero', ({ assert }) => {
        const data = [1.0, 2.0, 3.0, 4.0, 5.0]
        try {
            exponentialMovingAverage(data, 0)
            assert.fail('Expected error was not thrown')
        } catch (error) {
            assert.equal(error.message, 'Period must be greater than 0')
        }
    })

    test('test empty data', ({ assert }) => {
        const data = []
        try {
            exponentialMovingAverage(data, 3)
            assert.fail('Expected error was not thrown')
        } catch (error) {
            assert.equal(error.message, 'Prices vector must not be empty.')
        }
    })

    test('test period 1 returns raw data', ({ assert }) => {
        const data = [1.0, 2.0, 3.0, 4.0, 5.0]
        const result = exponentialMovingAverage(data, 1)
        assert.deepEqual(result, data)
    })

    test('test valid input', ({ assert }) => {
        const data = [1.0, 2.0, 3.0, 4.0, 5.0]
        // EMA now uses SMA seed: seed = (1+2+3)/3 = 2.0, k = 2/(3+1) = 0.5
        // Output length = data.length - period + 1 = 3
        // EMA[0] = SMA(1,2,3) = 2.0
        // EMA[1] = 0.5 * 4.0 + 0.5 * 2.0 = 3.0
        // EMA[2] = 0.5 * 5.0 + 0.5 * 3.0 = 4.0
        const expected = [2.0, 3.0, 4.0]
        const result = exponentialMovingAverage(data, 3)
        assert.deepEqual(result, expected)
    })

    test('test insufficient data returns array of NaNs', ({ assert }) => {
        const data = [1.0, 2.0] // length < period (3)
        try {
            exponentialMovingAverage(data, 3)
            assert.fail('Expected error was not thrown')
        } catch (error) {
            assert.equal(error.message, 'Period must be lower than data length')
        }
    })
})
