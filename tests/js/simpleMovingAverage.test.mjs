import { test } from '@japa/runner'
import { simpleMovingAverage } from '../../index.js'

test.group('SimpleMovingAverage', (group) => {

    test('test invalid period zero', ({ assert }) => {
        const data = [1.0, 2.0, 3.0]
        try {
            simpleMovingAverage(data, 0)
            assert.fail()
        } catch (error) {
            assert.equal(error.message, 'Period must be greater than 0')
        }
    })

    test('test empty data', ({ assert }) => {
        const data = []
        try {
            simpleMovingAverage(data, 2)
            assert.fail()
        } catch (error) {
            assert.equal(error.message, 'Data array cannot be empty')
        }
    })

    test('test data length less than period', ({ assert }) => {
        const data = [1.0, 2.0]
        try {
            simpleMovingAverage(data, 10)
            assert.fail()
        } catch (error) {
            assert.equal(error.message, 'Data array length (2) is less than period (10)')
        }
    })

    test('test valid input', ({ assert }) => {
        const data = [1.0, 2.0, 3.0, 4.0, 5.0];
        const result = simpleMovingAverage(data, 3);
        assert.deepEqual(result, [NaN, NaN, 2, 3, 4]);
    });

    /*
    test('should calculate SMA correctly', ({ assert }) => {
        const result = simpleMovingAverage([1.0, 2.0, 3.0, 4.0, 5.0, 6.0], 3)
        assert.equal(result[2], 2)
        assert.equal(result[3], 3)
        assert.equal(result[4], 4)
        assert.equal(result[5], 5)
    })
     */
})
