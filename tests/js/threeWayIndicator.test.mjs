import { test } from '@japa/runner'
import pkg from '../../index.js'
const { threeWayIndicator } = pkg
import { generateTestData } from './lib.mjs'

test.group('ThreeWayIndicator', (group) => {

    let data

    group.setup(() => {
        data = generateTestData(200)
    })

    test('all output arrays have the same length as input', ({ assert }) => {
        const result = threeWayIndicator(data)
        assert.equal(result.score.length, data.length)
        assert.equal(result.trend.length, data.length)
        assert.equal(result.momentum.length, data.length)
        assert.equal(result.volatility.length, data.length)
        assert.equal(result.signals.length, data.length)
    })

    test('score values are between -3 and 3', ({ assert }) => {
        const result = threeWayIndicator(data)
        for (const val of result.score) {
            if (!isNaN(val)) {
                assert.isTrue(
                    val >= -3 && val <= 3,
                    `Expected score to be between -3 and 3 but got ${val}`
                )
            }
        }
    })

    test('trend values are -1, 0, or 1', ({ assert }) => {
        const result = threeWayIndicator(data)
        const validValues = new Set([-1, 0, 1])
        for (const val of result.trend) {
            if (!isNaN(val)) {
                assert.isTrue(
                    validValues.has(val),
                    `Expected trend value to be -1, 0, or 1 but got ${val}`
                )
            }
        }
    })

    test('signals values are -1, 0, or 1', ({ assert }) => {
        const result = threeWayIndicator(data)
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
})
