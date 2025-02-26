import { test } from '@japa/runner'
import { directionalMovementIndex } from '../../index.js'
import { generateTestData } from "./lib.mjs";

test.group('Directional movement index', (group) => {

    test('test invalid period zero', ({ assert }) => {
        try {
            const data = generateTestData(10)
            directionalMovementIndex(data, 0)
            assert.fail()
        } catch (error) {
            assert.equal(error.message, 'Period must be greater than 0.')
        }
    })

    test('test empty data', ({ assert }) => {
        const data = []
        try {
            directionalMovementIndex(data, 14)
            assert.fail()
        } catch (error) {
            assert.equal(error.message, 'Not enough data points. Need at least 28')
        }
    })

    test('test with insufficient data', ({ assert }) => {
        const testData = generateTestData(3)
        try {
            directionalMovementIndex(testData, 10)
            assert.fail()
        } catch (error) {
            assert.equal(error.message, 'Not enough data points. Need at least 20')
        }
    })

    test('test invalid period', ({ assert }) => {
        const testData = generateTestData(30)
        try {
            directionalMovementIndex(testData, -1)
            assert.fail()
        } catch (error) {
            assert.equal(error.message, 'Period must be greater than 0.')
        }
    })

    test('test valid calculation', ({ assert }) => {
        const testData = generateTestData(30) // Doit être >= 2 * période
        const period = 14
        const result = directionalMovementIndex(testData, period)

        assert.lengthOf(result.plusDi, 30)
        assert.lengthOf(result.minusDi, 30)
        assert.lengthOf(result.adx, 30)

        assert.equal(result.adx.slice(0, period * 2 - 1).every(v => v === 0), true)
    })
})