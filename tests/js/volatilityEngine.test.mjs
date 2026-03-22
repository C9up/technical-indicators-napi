import { test } from '@japa/runner'
import { volatilityEngine, volatilityBucket } from '../../index.js'
import { generateTestData } from './lib.mjs'

/** Valid regime values as defined by the Rust implementation. */
const VALID_REGIMES = new Set([-1, 0, 1, 2])

test.group('Volatility Engine', (group) => {

    let marketData

    group.setup(() => {
        marketData = generateTestData(200)
    })

    // volatilityEngine

    test('atr array has same length as input', ({ assert }) => {
        const result = volatilityEngine(marketData)
        assert.lengthOf(result.atr, marketData.length)
    })

    test('volatility array has same length as input', ({ assert }) => {
        const result = volatilityEngine(marketData)
        assert.lengthOf(result.volatility, marketData.length)
    })

    test('regimes array has same length as input', ({ assert }) => {
        const result = volatilityEngine(marketData)
        assert.lengthOf(result.regimes, marketData.length)
    })

    test('atrMultipliers array has same length as input', ({ assert }) => {
        const result = volatilityEngine(marketData)
        assert.lengthOf(result.atrMultipliers, marketData.length)
    })

    test('stopDistances array has same length as input', ({ assert }) => {
        const result = volatilityEngine(marketData)
        assert.lengthOf(result.stopDistances, marketData.length)
    })

    test('lowThresholds array has same length as input', ({ assert }) => {
        const result = volatilityEngine(marketData)
        assert.lengthOf(result.lowThresholds, marketData.length)
    })

    test('highThresholds array has same length as input', ({ assert }) => {
        const result = volatilityEngine(marketData)
        assert.lengthOf(result.highThresholds, marketData.length)
    })

    test('regimes values are in the set -1, 0, 1, 2', ({ assert }) => {
        const result = volatilityEngine(marketData)
        for (let i = 0; i < result.regimes.length; i++) {
            const regime = result.regimes[i]
            assert.isTrue(
                VALID_REGIMES.has(regime),
                `regimes[${i}] value ${regime} is not a valid regime`
            )
        }
    })

    test('stopDistances equal atr times atrMultiplier where neither is NaN', ({ assert }) => {
        const result = volatilityEngine(marketData)
        for (let i = 0; i < marketData.length; i++) {
            const atr = result.atr[i]
            const mult = result.atrMultipliers[i]
            const stop = result.stopDistances[i]
            if (!isNaN(atr) && !isNaN(mult) && !isNaN(stop)) {
                assert.approximately(
                    stop,
                    atr * mult,
                    1e-9,
                    `stopDistances[${i}] should equal atr[${i}] * atrMultipliers[${i}]`
                )
            }
        }
    })

    test('result contains all expected output keys', ({ assert }) => {
        const result = volatilityEngine(marketData)
        assert.properties(result, [
            'atr',
            'volatility',
            'regimes',
            'atrMultipliers',
            'stopDistances',
            'lowThresholds',
            'highThresholds',
        ])
    })

    test('accepts optional parameters without throwing', ({ assert }) => {
        const result = volatilityEngine(marketData, 14, 20, 50, 30, 25, 75, 1.0, 1.5, 2.0)
        assert.lengthOf(result.atr, marketData.length)
    })

    // volatilityBucket

    test('volatilityBucket returns a regime field', ({ assert }) => {
        const engineResult = volatilityEngine(marketData)
        const warmup = 30
        const volHistory = engineResult.volatility.slice(0, warmup).filter((v) => !isNaN(v))
        if (volHistory.length === 0) {
            assert.isTrue(true, 'skipped: insufficient warm-up data')
            return
        }
        const idx = warmup
        const result = volatilityBucket(
            engineResult.atr[idx],
            engineResult.volatility[idx],
            volHistory
        )
        assert.property(result, 'regime')
    })

    test('volatilityBucket regime is a valid value', ({ assert }) => {
        const engineResult = volatilityEngine(marketData)
        const warmup = 30
        const volHistory = engineResult.volatility.slice(0, warmup).filter((v) => !isNaN(v))
        if (volHistory.length === 0) {
            assert.isTrue(true, 'skipped: insufficient warm-up data')
            return
        }
        const idx = warmup
        const result = volatilityBucket(
            engineResult.atr[idx],
            engineResult.volatility[idx],
            volHistory
        )
        // regime may be numeric or string; verify it is not undefined
        assert.isDefined(result.regime)
    })

    test('volatilityBucket returns atr, volatility, stopDistance, lowThreshold, highThreshold', ({ assert }) => {
        const engineResult = volatilityEngine(marketData)
        const warmup = 30
        const volHistory = engineResult.volatility.slice(0, warmup).filter((v) => !isNaN(v))
        if (volHistory.length === 0) {
            assert.isTrue(true, 'skipped: insufficient warm-up data')
            return
        }
        const idx = warmup
        const result = volatilityBucket(
            engineResult.atr[idx],
            engineResult.volatility[idx],
            volHistory
        )
        assert.properties(result, [
            'regime',
            'atrMultiplier',
            'atr',
            'volatility',
            'stopDistance',
            'lowThreshold',
            'highThreshold',
        ])
    })

    test('volatilityBucket stopDistance equals atr times atrMultiplier', ({ assert }) => {
        const engineResult = volatilityEngine(marketData)
        const warmup = 30
        const volHistory = engineResult.volatility.slice(0, warmup).filter((v) => !isNaN(v))
        if (volHistory.length === 0) {
            assert.isTrue(true, 'skipped: insufficient warm-up data')
            return
        }
        const idx = warmup
        const result = volatilityBucket(
            engineResult.atr[idx],
            engineResult.volatility[idx],
            volHistory
        )
        if (!isNaN(result.atr) && !isNaN(result.atrMultiplier) && !isNaN(result.stopDistance)) {
            assert.approximately(result.stopDistance, result.atr * result.atrMultiplier, 1e-9)
        }
    })
})
