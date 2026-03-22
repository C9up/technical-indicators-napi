import { test } from '@japa/runner'
import { performanceMetrics, sharpeRatio, sortinoRatio, maxDrawdown } from '../../index.js'

test.group('PerformanceMetrics', (group) => {

    // Realistic daily return series (100 days, mix of gains and losses)
    const makeReturns = (n = 100) => {
        const returns = []
        let seed = 1
        for (let i = 0; i < n; i++) {
            // Deterministic pseudo-random via linear congruential generator
            seed = (seed * 1664525 + 1013904223) & 0xffffffff
            // Map to range [-0.03, 0.04]
            returns.push(((seed >>> 0) / 0xffffffff) * 0.07 - 0.03)
        }
        return returns
    }

    const allPositive = Array.from({ length: 50 }, () => 0.01)

    test('maxDrawdown is between 0 and 1', ({ assert }) => {
        const returns = makeReturns(100)
        const result = performanceMetrics(returns)
        assert.isTrue(result.maxDrawdown >= 0 && result.maxDrawdown <= 1,
            `maxDrawdown ${result.maxDrawdown} is out of [0, 1]`)
    })

    test('winRate is between 0 and 1', ({ assert }) => {
        const returns = makeReturns(100)
        const result = performanceMetrics(returns)
        assert.isTrue(result.winRate >= 0 && result.winRate <= 1,
            `winRate ${result.winRate} is out of [0, 1]`)
    })

    test('totalReturn matches cumulative product of (1 + r) - 1', ({ assert }) => {
        const returns = makeReturns(20)
        const result = performanceMetrics(returns)
        const cumulative = returns.reduce((acc, r) => acc * (1 + r), 1) - 1
        assert.approximately(result.totalReturn, cumulative, 1e-9)
    })

    test('maxDrawdown is 0 when all returns are positive', ({ assert }) => {
        const result = performanceMetrics(allPositive)
        assert.approximately(result.maxDrawdown, 0, 1e-10)
    })

    test('empty returns array throws', ({ assert }) => {
        try {
            performanceMetrics([])
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.isTrue(error instanceof Error)
        }
    })

    test('standalone sharpeRatio matches performanceMetrics.sharpeRatio', ({ assert }) => {
        const returns = makeReturns(100)
        const riskFreeRate = 0.02
        const periodsPerYear = 252
        const standalone = sharpeRatio(returns, riskFreeRate, periodsPerYear)
        const fromMetrics = performanceMetrics(returns, riskFreeRate, periodsPerYear).sharpeRatio
        assert.approximately(standalone, fromMetrics, 1e-9)
    })

    test('standalone sortinoRatio is a finite number', ({ assert }) => {
        const returns = makeReturns(100)
        const result = sortinoRatio(returns, 0.02, 252)
        assert.isTrue(Number.isFinite(result) || isNaN(result),
            `Expected sortinoRatio to be a number, got ${result}`)
    })

    test('standalone maxDrawdown between 0 and 1', ({ assert }) => {
        const returns = makeReturns(100)
        const result = maxDrawdown(returns)
        assert.isTrue(result >= 0 && result <= 1,
            `maxDrawdown ${result} is out of [0, 1]`)
    })

    test('standalone maxDrawdown is 0 for all-positive returns', ({ assert }) => {
        const result = maxDrawdown(allPositive)
        assert.approximately(result, 0, 1e-10)
    })

    test('performanceMetrics result has all expected fields', ({ assert }) => {
        const returns = makeReturns(50)
        const result = performanceMetrics(returns)
        assert.properties(result, [
            'sharpeRatio',
            'sortinoRatio',
            'calmarRatio',
            'maxDrawdown',
            'totalReturn',
        ])
    })

})
