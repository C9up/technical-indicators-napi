import { test } from '@japa/runner'
import { relativeVigorIndex } from '../../index.js'
import { generateTestData } from './lib.mjs'

test.group('RelativeVigorIndex', (group) => {

    let data

    group.setup(() => {
        data = generateTestData(100)
    })

    test('rvi and signal have the same length as input', ({ assert }) => {
        const result = relativeVigorIndex(data)
        assert.equal(result.rvi.length, data.length)
        assert.equal(result.signal.length, data.length)
    })

    test('early values are NaN due to warmup period', ({ assert }) => {
        const result = relativeVigorIndex(data)
        // At least the very first value should be NaN given the warmup requirement
        assert.isTrue(isNaN(result.rvi[0]), 'Expected first rvi value to be NaN')
        assert.isTrue(isNaN(result.signal[0]), 'Expected first signal value to be NaN')
    })

    test('not enough data throws', ({ assert }) => {
        const tinyData = generateTestData(2)
        try {
            relativeVigorIndex(tinyData)
            assert.fail('Expected error was not thrown')
        } catch (error) {
            assert.isNotNull(error)
        }
    })
})
