import { test } from '@japa/runner'
import { exponentialMovingAverage } from '../../index.js'

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
        // Pour une période de 3, le facteur de lissage est 2/(3+1)=0.5.
        // Calcul étape par étape :
        // EMA[0] = 1.0
        // EMA[1] = 0.5 * 2.0 + 0.5 * 1.0 = 1.5
        // EMA[2] = 0.5 * 3.0 + 0.5 * 1.5 = 2.25
        // EMA[3] = 0.5 * 4.0 + 0.5 * 2.25 = 3.125
        // EMA[4] = 0.5 * 5.0 + 0.5 * 3.125 = 4.0625
        const expected = [1.0, 1.5, 2.25, 3.125, 4.0625]
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
