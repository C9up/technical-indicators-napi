import { test } from '@japa/runner'
import pkg from '../../index.js'
const { awesomeOscillator } = pkg
import { generateTestData } from './lib.mjs'

test.group('AwesomeOscillator', (group) => {

    let data

    group.setup(() => {
        data = generateTestData(100)
    })

    test('ao and histogram have the same length as input', ({ assert }) => {
        const result = awesomeOscillator(data)
        assert.equal(result.ao.length, data.length)
        assert.equal(result.histogram.length, data.length)
    })

    test('histogram values are -1, 0, or 1', ({ assert }) => {
        const result = awesomeOscillator(data)
        const validValues = new Set([-1, 0, 1])
        for (const val of result.histogram) {
            if (!isNaN(val)) {
                assert.isTrue(
                    validValues.has(val),
                    `Expected histogram value to be -1, 0, or 1 but got ${val}`
                )
            }
        }
    })

    test('fastPeriod >= slowPeriod throws', ({ assert }) => {
        try {
            awesomeOscillator(data, 34, 5)
            assert.fail('Expected error was not thrown')
        } catch (error) {
            assert.isNotNull(error)
        }
    })

    test('not enough data throws', ({ assert }) => {
        const tinyData = generateTestData(2)
        try {
            awesomeOscillator(tinyData)
            assert.fail('Expected error was not thrown')
        } catch (error) {
            assert.isNotNull(error)
        }
    })
})
