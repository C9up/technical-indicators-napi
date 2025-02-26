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
            assert.equal(error.message, 'Not enough data points')
        }
    })

    test('test with insufficient data', ({ assert }) => {
        const testData = generateTestData(3)
        try {
            directionalMovementIndex(testData, 10)
            assert.fail()
        } catch (error) {
            assert.equal(error.message, 'Not enough data points')
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
})
