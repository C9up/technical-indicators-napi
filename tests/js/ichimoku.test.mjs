import { test } from '@japa/runner'
import { ichimoku, lowHighOpenCloseVolumeDateToArray } from '../../index.js'
import { generateTestData } from './lib.mjs'

test.group('Ichimoku Cloud Indicator', () => {

    test('should calculate basic Ichimoku values', ({ assert }) => {
        // Generate 100 days of data to ensure enough points for all calculations
        const testData = generateTestData(100)
        const result = ichimoku(testData)
        assert.properties(result[0], [
            'tenkanSen',
            'kijunSen',
            'senkouSpanA',
            'senkouSpanB',
            'chikouSpan'
        ])

        // Check array lengths
        assert.equal(result.length, testData.length)
    })

    test('should handle minimal data set', ({ assert }) => {
        const minimalData = generateTestData(53) // Minimum required for 52-period calculations
        const result = ichimoku(minimalData)

        // Verify initial values are NaN
        assert.isTrue(isNaN(result[0].tenkanSen))
        assert.isTrue(isNaN(result[0].kijunSen))
        assert.isTrue(isNaN(result[0].senkouSpanA))
        assert.isTrue(isNaN(result[0].senkouSpanB))
    })

    test('should calculate correct Tenkan-sen values', ({ assert }) => {
        const testData = generateTestData(100)

        const lhocvd = lowHighOpenCloseVolumeDateToArray(testData)
        const result = ichimoku(testData)

        // First 8 values should be NaN (9-period calculation)
        for (let i = 0; i < 8; i++) {
            assert.isTrue(isNaN(result[i].tenkanSen))
        }

        const expectedMax = Math.max(...lhocvd.highs.slice(0, 9));
        const expectedMin = Math.min(...lhocvd.lows.slice(0, 9));
        const expectedTenkan = (expectedMax + expectedMin) / 2;

        assert.approximately(result[8].tenkanSen, expectedTenkan, 0.01);
    })

    test('should verify Senkou Span displacement', ({ assert }) => {
        const testData = generateTestData(60)
        const result = ichimoku(testData)

        // First 26 values should be NaN due to displacement
        for (let i = 0; i < 26; i++) {
            assert.isTrue(isNaN(result[i].senkouSpanA))
            assert.isTrue(isNaN(result[i].senkouSpanB))
        }
    })

    test('should calculate correct Chikou Span', ({ assert }) => {
        const chikouShift = 26
        const testData = generateTestData(80)
        const result = ichimoku(testData)

        // chikou[i] = closes[i + chikouShift] (forward-looking)
        // So the last chikouShift values will be NaN (no future data)
        for (let i = testData.length - chikouShift; i < testData.length; i++) {
            assert.isTrue(Number.isNaN(result[i].chikouSpan),
                `chikouSpan[${i}] should be NaN`)
        }

        // Values at index i should equal the close price chikouShift bars ahead
        for (let i = 0; i < testData.length - chikouShift; i++) {
            assert.equal(result[i].chikouSpan, testData[i + chikouShift].close,
                `chikouSpan[${i}] should equal close[${i + chikouShift}]`)
        }
    })
})