import { test } from '@japa/runner'
import { patternMemory } from '../../index.js'
import { generateTestData } from './lib.mjs'

test.group('PatternMemory', (group) => {

    let data
    const kNeighbors = 5

    group.setup(() => {
        data = generateTestData(200)
    })

    test('all output arrays have the same length as input', ({ assert }) => {
        const result = patternMemory(data, kNeighbors)
        assert.equal(result.signal.length, data.length)
        assert.equal(result.normalizedSignal.length, data.length)
        assert.equal(result.bullishCount.length, data.length)
        assert.equal(result.bearishCount.length, data.length)
        assert.equal(result.avgDistance.length, data.length)
    })

    test('normalizedSignal values are between -1 and 1 where not NaN', ({ assert }) => {
        const result = patternMemory(data, kNeighbors)
        for (const val of result.normalizedSignal) {
            if (!isNaN(val)) {
                assert.isTrue(
                    val >= -1 && val <= 1,
                    `Expected normalizedSignal to be between -1 and 1 but got ${val}`
                )
            }
        }
    })

    test('bullishCount[i] + bearishCount[i] <= kNeighbors', ({ assert }) => {
        const result = patternMemory(data, kNeighbors)
        for (let i = 0; i < data.length; i++) {
            const bullish = result.bullishCount[i]
            const bearish = result.bearishCount[i]
            if (!isNaN(bullish) && !isNaN(bearish)) {
                assert.isTrue(
                    bullish + bearish <= kNeighbors,
                    `Expected bullishCount + bearishCount <= ${kNeighbors} at index ${i}, got ${bullish + bearish}`
                )
            }
        }
    })

    test('not enough data throws', ({ assert }) => {
        const tinyData = generateTestData(2)
        try {
            patternMemory(tinyData)
            assert.fail('Expected error was not thrown')
        } catch (error) {
            assert.isNotNull(error)
        }
    })
})
