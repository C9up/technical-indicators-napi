import { test } from '@japa/runner'
import { generateTestData } from './lib.mjs';
import {kagiChart} from '../../index.js'

test.group('Kagi Chart', () => {

    test('should generate the correct Kagi chart data for a given price series', ({ assert }) => {
        // Generating random test data for 30 days
        const testData = generateTestData(30);

        // Extracting only the 'close' prices for Kagi chart input
        const prices = testData.map(item => item.close);

        // Define a reversal amount (e.g., 1.0)
        const reversalAmount = 1.0;

        // Call the wasm function
        const result = kagiChart(prices, reversalAmount);

        // Assert that the result contains prices and directions
        assert.exists(result)

        result.forEach((price, index) => {
            if (index > 0) {
                assert.exists(price.price)
                assert.exists(price.direction)
                assert.isAbove(price.price, 0)
                assert.oneOf(price.direction, ['Yang', 'Yin'])
            }
        })
    });

    test('should handle an empty price list gracefully', ({ assert }) => {
        try {
            kagiChart([], 1.0);
            assert.fail()
        } catch (error) {
            assert.equal(error.message, 'Prices vector must not be empty.')
        }
    });
});
