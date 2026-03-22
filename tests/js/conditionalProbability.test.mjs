import { test } from '@japa/runner'
import { conditionalProbability, conditionalProbabilityMatrix } from '../../index.js'
import { generateTestData } from './lib.mjs'

test.group('Conditional Probability', (group) => {

    let prices

    group.setup(() => {
        const data = generateTestData(200)
        prices = data.map((d) => d.close)
    })

    test('valid prices return upProbability between 0 and 1', ({ assert }) => {
        const result = conditionalProbability(prices, 5, 5, 0.02, 0.02)
        assert.isAtLeast(result.upProbability, 0)
        assert.isAtMost(result.upProbability, 1)
    })

    test('valid prices return downProbability between 0 and 1', ({ assert }) => {
        const result = conditionalProbability(prices, 5, 5, 0.02, 0.02)
        assert.isAtLeast(result.downProbability, 0)
        assert.isAtMost(result.downProbability, 1)
    })

    test('firstMoveCount is greater than or equal to upCount plus downCount', ({ assert }) => {
        const result = conditionalProbability(prices, 5, 5, 0.02, 0.02)
        assert.isAtLeast(result.firstMoveCount, result.upCount + result.downCount)
    })

    test('result contains all expected fields', ({ assert }) => {
        const result = conditionalProbability(prices, 5, 5, 0.02, 0.02)
        assert.properties(result, [
            'upProbability',
            'downProbability',
            'firstMoveCount',
            'upCount',
            'downCount',
            'upIndices',
            'downIndices',
            'secondMoveReturns',
        ])
    })

    test('upIndices and downIndices are arrays', ({ assert }) => {
        const result = conditionalProbability(prices, 5, 5, 0.02, 0.02)
        assert.isArray(result.upIndices)
        assert.isArray(result.downIndices)
    })

    test('upCount matches length of upIndices', ({ assert }) => {
        const result = conditionalProbability(prices, 5, 5, 0.02, 0.02)
        assert.equal(result.upCount, result.upIndices.length)
    })

    test('downCount matches length of downIndices', ({ assert }) => {
        const result = conditionalProbability(prices, 5, 5, 0.02, 0.02)
        assert.equal(result.downCount, result.downIndices.length)
    })

    test('secondMoveReturns is an array', ({ assert }) => {
        const result = conditionalProbability(prices, 5, 5, 0.02, 0.02)
        assert.isArray(result.secondMoveReturns)
    })

    test('empty prices throws', ({ assert }) => {
        try {
            conditionalProbability([], 5, 5, 0.02, 0.02)
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.instanceOf(error, Error)
        }
    })

    test('zero firstMoveDays throws', ({ assert }) => {
        try {
            conditionalProbability(prices, 0, 5, 0.02, 0.02)
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.instanceOf(error, Error)
        }
    })

    test('zero secondMoveDays throws', ({ assert }) => {
        try {
            conditionalProbability(prices, 5, 0, 0.02, 0.02)
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.instanceOf(error, Error)
        }
    })

    test('matrix returns correct number of entries', ({ assert }) => {
        const firstThresholds = [0.01, 0.02, 0.03]
        const secondThresholds = [0.01, 0.02]
        const result = conditionalProbabilityMatrix(prices, 5, 5, firstThresholds, secondThresholds)
        assert.isArray(result)
        assert.lengthOf(result, firstThresholds.length * secondThresholds.length)
    })

    test('matrix entries contain expected fields', ({ assert }) => {
        const firstThresholds = [0.02]
        const secondThresholds = [0.02]
        const result = conditionalProbabilityMatrix(prices, 5, 5, firstThresholds, secondThresholds)
        assert.properties(result[0], [
            'firstThreshold',
            'secondThreshold',
            'upProbability',
            'downProbability',
            'sampleCount',
        ])
    })

    test('matrix entry probabilities are between 0 and 1', ({ assert }) => {
        const firstThresholds = [0.01, 0.03, 0.05]
        const secondThresholds = [0.01, 0.03]
        const result = conditionalProbabilityMatrix(prices, 5, 5, firstThresholds, secondThresholds)
        for (const entry of result) {
            assert.isAtLeast(entry.upProbability, 0)
            assert.isAtMost(entry.upProbability, 1)
            assert.isAtLeast(entry.downProbability, 0)
            assert.isAtMost(entry.downProbability, 1)
        }
    })

    test('matrix reflects the supplied threshold values', ({ assert }) => {
        const firstThresholds = [0.01, 0.05]
        const secondThresholds = [0.02]
        const result = conditionalProbabilityMatrix(prices, 5, 5, firstThresholds, secondThresholds)
        const firstThresholdValues = result.map((e) => e.firstThreshold)
        assert.include(firstThresholdValues, 0.01)
        assert.include(firstThresholdValues, 0.05)
    })

    test('matrix empty prices throws', ({ assert }) => {
        try {
            conditionalProbabilityMatrix([], 5, 5, [0.02], [0.02])
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.instanceOf(error, Error)
        }
    })

    test('matrix zero firstMoveDays throws', ({ assert }) => {
        try {
            conditionalProbabilityMatrix(prices, 0, 5, [0.02], [0.02])
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.instanceOf(error, Error)
        }
    })

    test('matrix zero secondMoveDays throws', ({ assert }) => {
        try {
            conditionalProbabilityMatrix(prices, 5, 0, [0.02], [0.02])
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.instanceOf(error, Error)
        }
    })
})
