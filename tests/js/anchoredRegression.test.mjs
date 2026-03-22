import { test } from '@japa/runner'
import { anchoredRegressionStatic, anchoredRegressionRolling } from '../../index.js'
import { generateTestData } from './lib.mjs'

test.group('Anchored Regression', (group) => {

    let prices

    group.setup(() => {
        const data = generateTestData(100)
        prices = data.map((d) => d.close)
    })

    // anchoredRegressionStatic

    test('anchoredRegressionStatic fitted has same length as input', ({ assert }) => {
        const result = anchoredRegressionStatic(prices, 20)
        assert.lengthOf(result.fitted, prices.length)
    })

    test('anchoredRegressionStatic upperBand has same length as input', ({ assert }) => {
        const result = anchoredRegressionStatic(prices, 20)
        assert.lengthOf(result.upperBand, prices.length)
    })

    test('anchoredRegressionStatic lowerBand has same length as input', ({ assert }) => {
        const result = anchoredRegressionStatic(prices, 20)
        assert.lengthOf(result.lowerBand, prices.length)
    })

    test('anchoredRegressionStatic slopes has same length as input', ({ assert }) => {
        const result = anchoredRegressionStatic(prices, 20)
        assert.lengthOf(result.slopes, prices.length)
    })

    test('anchoredRegressionStatic segments cover all bars', ({ assert }) => {
        const result = anchoredRegressionStatic(prices, 20)
        assert.isArray(result.segments)
        const coveredIndices = new Set()
        for (const segment of result.segments) {
            for (let i = segment.startIndex; i <= segment.endIndex; i++) {
                coveredIndices.add(i)
            }
        }
        for (let i = 0; i < prices.length; i++) {
            assert.isTrue(coveredIndices.has(i), `Bar ${i} not covered by any segment`)
        }
    })

    test('anchoredRegressionStatic upperBand >= fitted >= lowerBand where values are not NaN', ({ assert }) => {
        const result = anchoredRegressionStatic(prices, 20)
        for (let i = 0; i < prices.length; i++) {
            const u = result.upperBand[i]
            const f = result.fitted[i]
            const l = result.lowerBand[i]
            if (!isNaN(u) && !isNaN(f) && !isNaN(l)) {
                assert.isAtLeast(u, f, `upperBand[${i}] should be >= fitted[${i}]`)
                assert.isAtLeast(f, l, `fitted[${i}] should be >= lowerBand[${i}]`)
            }
        }
    })

    test('anchoredRegressionStatic anchorPeriod less than 2 throws', ({ assert }) => {
        try {
            anchoredRegressionStatic(prices, 1)
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.instanceOf(error, Error)
        }
    })

    test('anchoredRegressionStatic empty data throws', ({ assert }) => {
        try {
            anchoredRegressionStatic([], 20)
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.instanceOf(error, Error)
        }
    })

    test('anchoredRegressionStatic accepts optional bandMult', ({ assert }) => {
        const result = anchoredRegressionStatic(prices, 20, 2.0)
        assert.lengthOf(result.fitted, prices.length)
    })

    // anchoredRegressionRolling

    test('anchoredRegressionRolling fitted has same length as input', ({ assert }) => {
        const result = anchoredRegressionRolling(prices, 20)
        assert.lengthOf(result.fitted, prices.length)
    })

    test('anchoredRegressionRolling upperBand has same length as input', ({ assert }) => {
        const result = anchoredRegressionRolling(prices, 20)
        assert.lengthOf(result.upperBand, prices.length)
    })

    test('anchoredRegressionRolling lowerBand has same length as input', ({ assert }) => {
        const result = anchoredRegressionRolling(prices, 20)
        assert.lengthOf(result.lowerBand, prices.length)
    })

    test('anchoredRegressionRolling slopes has same length as input', ({ assert }) => {
        const result = anchoredRegressionRolling(prices, 20)
        assert.lengthOf(result.slopes, prices.length)
    })

    test('anchoredRegressionRolling segments cover all bars', ({ assert }) => {
        const result = anchoredRegressionRolling(prices, 20)
        assert.isArray(result.segments)
        const coveredIndices = new Set()
        for (const segment of result.segments) {
            for (let i = segment.startIndex; i <= segment.endIndex; i++) {
                coveredIndices.add(i)
            }
        }
        for (let i = 0; i < prices.length; i++) {
            assert.isTrue(coveredIndices.has(i), `Bar ${i} not covered by any segment`)
        }
    })

    test('anchoredRegressionRolling upperBand >= fitted >= lowerBand where values are not NaN', ({ assert }) => {
        const result = anchoredRegressionRolling(prices, 20)
        for (let i = 0; i < prices.length; i++) {
            const u = result.upperBand[i]
            const f = result.fitted[i]
            const l = result.lowerBand[i]
            if (!isNaN(u) && !isNaN(f) && !isNaN(l)) {
                assert.isAtLeast(u, f, `upperBand[${i}] should be >= fitted[${i}]`)
                assert.isAtLeast(f, l, `fitted[${i}] should be >= lowerBand[${i}]`)
            }
        }
    })

    test('anchoredRegressionRolling anchorPeriod less than 2 throws', ({ assert }) => {
        try {
            anchoredRegressionRolling(prices, 1)
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.instanceOf(error, Error)
        }
    })

    test('anchoredRegressionRolling empty data throws', ({ assert }) => {
        try {
            anchoredRegressionRolling([], 20)
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.instanceOf(error, Error)
        }
    })

    test('anchoredRegressionRolling accepts optional bandMult', ({ assert }) => {
        const result = anchoredRegressionRolling(prices, 20, 1.5)
        assert.lengthOf(result.fitted, prices.length)
    })
})
