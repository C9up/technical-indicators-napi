import { test } from '@japa/runner'
import { harVolatility } from '../../index.js'
import { generateTestData } from './lib.mjs'

test.group('HarVolatility', (group) => {

    let data

    group.setup(() => {
        data = generateTestData(300)
    })

    test('all output arrays have the same length as input', ({ assert }) => {
        const result = harVolatility(data)
        assert.equal(result.predictedVol.length, data.length)
        assert.equal(result.volDaily.length, data.length)
        assert.equal(result.volWeekly.length, data.length)
        assert.equal(result.volMonthly.length, data.length)
        assert.equal(result.regime.length, data.length)
        assert.equal(result.exposure.length, data.length)
    })

    test('regime values are -1, 0, 1, or 2', ({ assert }) => {
        const result = harVolatility(data)
        const validValues = new Set([-1, 0, 1, 2])
        for (const val of result.regime) {
            if (!isNaN(val)) {
                assert.isTrue(
                    validValues.has(val),
                    `Expected regime value to be -1, 0, 1, or 2 but got ${val}`
                )
            }
        }
    })

    test('exposure values are 0.0, 1.0, or 2.0 where not NaN', ({ assert }) => {
        const result = harVolatility(data)
        const validValues = new Set([0.0, 1.0, 2.0])
        for (const val of result.exposure) {
            if (!isNaN(val)) {
                assert.isTrue(
                    validValues.has(val),
                    `Expected exposure value to be 0.0, 1.0, or 2.0 but got ${val}`
                )
            }
        }
    })

    test('not enough data throws', ({ assert }) => {
        const tinyData = generateTestData(2)
        try {
            harVolatility(tinyData)
            assert.fail('Expected error was not thrown')
        } catch (error) {
            assert.isNotNull(error)
        }
    })
})
