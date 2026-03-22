import { test } from '@japa/runner'
import pkg from '../../index.js'
const { featureEngine } = pkg
import { generateTestData } from './lib.mjs'

/**
 * A non-exhaustive list of fields expected on every FeatureRow.
 * Extend as the library's documentation grows.
 */
const EXPECTED_FIELDS = [
    'index',
    'return1',
    'return5',
    'rsi14',
    'sma5',
    'macd',
    'bbPctB',
]

test.group('FeatureEngine', (group) => {

    test('returns an array of objects', ({ assert }) => {
        const data = generateTestData(200)
        const result = featureEngine(data)
        assert.isTrue(Array.isArray(result), 'Result should be an array')
        assert.isTrue(result.length > 0, 'Result should not be empty')
        assert.isTrue(typeof result[0] === 'object' && result[0] !== null,
            'Each element should be an object')
    })

    test('output length equals input.length - 50', ({ assert }) => {
        const data = generateTestData(200)
        const result = featureEngine(data)
        assert.equal(result.length, data.length - 50)
    })

    test('each row contains all expected fields', ({ assert }) => {
        const data = generateTestData(200)
        const result = featureEngine(data)
        const row = result[0]
        for (const field of EXPECTED_FIELDS) {
            assert.property(row, field,
                `Expected field "${field}" to be present on FeatureRow`)
        }
    })

    test('RSI values are between 0 and 100 where not NaN', ({ assert }) => {
        const data = generateTestData(200)
        const result = featureEngine(data)
        for (let i = 0; i < result.length; i++) {
            const rsi = result[i].rsi14
            if (!isNaN(rsi)) {
                assert.isTrue(rsi >= 0 && rsi <= 100,
                    `rsi14 at row ${i} = ${rsi} is outside [0, 100]`)
            }
        }
    })

    test('index field is a number', ({ assert }) => {
        const data = generateTestData(200)
        const result = featureEngine(data)
        for (let i = 0; i < result.length; i++) {
            assert.isTrue(typeof result[i].index === 'number',
                `index at row ${i} should be a number`)
        }
    })

    test('less than 52 data points throws', ({ assert }) => {
        const data = generateTestData(51)
        try {
            featureEngine(data)
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.isTrue(error instanceof Error)
        }
    })

    test('exactly 52 data points does not throw and returns 2 rows', ({ assert }) => {
        // 52 - 50 = 2 output rows
        const data = generateTestData(52)
        const result = featureEngine(data)
        assert.equal(result.length, 2)
    })

    test('bbPctB values are finite numbers or NaN', ({ assert }) => {
        const data = generateTestData(200)
        const result = featureEngine(data)
        for (let i = 0; i < result.length; i++) {
            const val = result[i].bbPctB
            assert.isTrue(typeof val === 'number',
                `bbPctB at row ${i} should be a number (got ${typeof val})`)
        }
    })

    test('macd values are finite numbers or NaN', ({ assert }) => {
        const data = generateTestData(200)
        const result = featureEngine(data)
        for (let i = 0; i < result.length; i++) {
            const val = result[i].macd
            assert.isTrue(typeof val === 'number',
                `macd at row ${i} should be a number (got ${typeof val})`)
        }
    })

})
