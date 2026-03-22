import { test } from '@japa/runner'
import { spreadEstimator, rollSpreadEstimator, corwinSchultzSpreadEstimator } from '../../index.js'
import { generateTestData } from './lib.mjs'

test.group('Spread Estimator', (group) => {

    let marketData
    let prices

    group.setup(() => {
        marketData = generateTestData(100)
        prices = marketData.map((d) => d.close)
    })

    // spreadEstimator

    test('spreadEstimator returns spreads array with correct length', ({ assert }) => {
        const window = 10
        const result = spreadEstimator(marketData, window)
        assert.isArray(result.spreads)
        assert.lengthOf(result.spreads, marketData.length)
    })

    test('spreadEstimator returns signedSpreads array with correct length', ({ assert }) => {
        const window = 10
        const result = spreadEstimator(marketData, window)
        assert.isArray(result.signedSpreads)
        assert.lengthOf(result.signedSpreads, marketData.length)
    })

    test('spreadEstimator non-NaN spreads are non-negative', ({ assert }) => {
        const window = 10
        const result = spreadEstimator(marketData, window)
        for (const value of result.spreads) {
            if (!isNaN(value)) {
                assert.isAtLeast(value, 0)
            }
        }
    })

    test('spreadEstimator result contains spreads and signedSpreads keys', ({ assert }) => {
        const result = spreadEstimator(marketData, 10)
        assert.properties(result, ['spreads', 'signedSpreads'])
    })

    test('spreadEstimator window less than 2 throws', ({ assert }) => {
        try {
            spreadEstimator(marketData, 1)
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.instanceOf(error, Error)
        }
    })

    test('spreadEstimator not enough data throws', ({ assert }) => {
        try {
            spreadEstimator(generateTestData(1), 10)
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.instanceOf(error, Error)
        }
    })

    // rollSpreadEstimator

    test('rollSpreadEstimator returns array with correct length', ({ assert }) => {
        const window = 10
        const result = rollSpreadEstimator(prices, window)
        assert.isArray(result)
        assert.lengthOf(result, prices.length - 1)
    })

    test('rollSpreadEstimator non-NaN values are non-negative', ({ assert }) => {
        const window = 10
        const result = rollSpreadEstimator(prices, window)
        for (const value of result) {
            if (!isNaN(value)) {
                assert.isAtLeast(value, 0)
            }
        }
    })

    test('rollSpreadEstimator window less than 2 throws', ({ assert }) => {
        try {
            rollSpreadEstimator(prices, 1)
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.instanceOf(error, Error)
        }
    })

    test('rollSpreadEstimator not enough data throws', ({ assert }) => {
        try {
            rollSpreadEstimator([100], 10)
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.instanceOf(error, Error)
        }
    })

    // corwinSchultzSpreadEstimator

    test('corwinSchultzSpreadEstimator returns array with correct length', ({ assert }) => {
        const window = 10
        const result = corwinSchultzSpreadEstimator(marketData, window)
        assert.isArray(result)
        assert.lengthOf(result, marketData.length)
    })

    test('corwinSchultzSpreadEstimator non-NaN values are non-negative', ({ assert }) => {
        const window = 10
        const result = corwinSchultzSpreadEstimator(marketData, window)
        for (const value of result) {
            if (!isNaN(value)) {
                assert.isAtLeast(value, 0)
            }
        }
    })

    test('corwinSchultzSpreadEstimator window less than 3 throws', ({ assert }) => {
        try {
            corwinSchultzSpreadEstimator(marketData, 2)
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.instanceOf(error, Error)
        }
    })

    test('corwinSchultzSpreadEstimator not enough data throws', ({ assert }) => {
        try {
            corwinSchultzSpreadEstimator(generateTestData(1), 10)
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.instanceOf(error, Error)
        }
    })
})
