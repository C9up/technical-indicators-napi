import { test } from '@japa/runner'
import { bollingerBands } from '../../index.js'

test.group('BollingerBands', (group) => {

    test('test invalid period zero', ({ assert }) => {
        const data = [1.0, 2.0, 3.0, 4.0, 5.0]
        try {
            bollingerBands(data, 0)
            assert.fail()
        } catch (error) {
            assert.equal(error.message, 'Period must be greater than 0.')
        }
    })

    test('test empty data', ({ assert }) => {
        const data = []
        try {
            bollingerBands(data, 20)
            assert.fail()
        } catch (error) {
            assert.equal(error.message, 'Prices vector must not be empty.')
        }
    })

    test('test valid input', ({ assert }) => {
        const data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
        const result = bollingerBands(data, 3, 2)

        assert.deepEqual(result.middle, [2, 3, 4, 5, 6, 7, 8, 9])
        assert.deepEqual(result.upper, [
            3.632993161855452,
            4.6329931618554525,
            5.6329931618554525,
            6.6329931618554525,
            7.6329931618554525,
            8.632993161855453,
            9.632993161855453,
            10.632993161855453
        ])
        assert.deepEqual(result.lower, [
            0.36700683814454793,
            1.367006838144548,
            2.367006838144548,
            3.367006838144548,
            4.3670068381445475,
            5.3670068381445475,
            6.3670068381445475,
            7.3670068381445475
        ])
    })

    test('test with default multiplier', ({ assert }) => {
        const data = [1.0, 2.0, 3.0, 4.0, 5.0]
        const expected = {
            middle: [2, 3, 4],
            upper: [3.632993161855452, 4.6329931618554525, 5.6329931618554525],
            lower: [0.36700683814454793, 1.367006838144548, 2.367006838144548],
        }
        const result = bollingerBands(data, 3)
        assert.deepEqual(result, expected)
    })

    test('test invalid multiplier', ({ assert }) => {
        const data = [1.0, 2.0, 3.0, 4.0, 5.0]
        try {
            bollingerBands(data, 3, -1)
            assert.fail()
        } catch (error) {
            assert.equal(error.message, 'Multiplier must be greater than 0.')
        }
    })

    test('test invalid period', ({ assert }) => {
        const data = [1.0, 2.0, 3.0, 4.0, 5.0]
        try {
            bollingerBands(data, -1)
            assert.fail()
        } catch (error) {
            assert.equal(error.message, 'Period must be greater than 0.')
        }
    })
})