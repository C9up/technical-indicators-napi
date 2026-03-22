import { test } from '@japa/runner'
import { choppinessIndex } from '../../index.js'
import { generateTestData } from './lib.mjs'

test.group('ChoppinessIndex', (group) => {

    let data

    group.setup(() => {
        data = generateTestData(100)
    })

    test('chop and signals have the same length as input', ({ assert }) => {
        const result = choppinessIndex(data)
        assert.equal(result.chop.length, data.length)
        assert.equal(result.signals.length, data.length)
    })

    test('non-NaN chop values are between 0 and 100', ({ assert }) => {
        const result = choppinessIndex(data)
        for (const val of result.chop) {
            if (!isNaN(val)) {
                assert.isTrue(
                    val >= 0 && val <= 100,
                    `Expected chop value to be between 0 and 100 but got ${val}`
                )
            }
        }
    })

    test('signals values are -1, 0, or 1', ({ assert }) => {
        const result = choppinessIndex(data)
        const validValues = new Set([-1, 0, 1])
        for (const val of result.signals) {
            if (!isNaN(val)) {
                assert.isTrue(
                    validValues.has(val),
                    `Expected signals value to be -1, 0, or 1 but got ${val}`
                )
            }
        }
    })

    test('not enough data throws', ({ assert }) => {
        const tinyData = generateTestData(2)
        try {
            choppinessIndex(tinyData)
            assert.fail('Expected error was not thrown')
        } catch (error) {
            assert.isNotNull(error)
        }
    })
})
