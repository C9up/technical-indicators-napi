import { test } from '@japa/runner'
import pkg from '../../index.js'
const { yangZhangVolatility } = pkg
import { generateTestData } from './lib.mjs'

test.group('YangZhangVolatility', (group) => {

    let data

    group.setup(() => {
        data = generateTestData(100)
    })

    test('all output arrays have the same length as input', ({ assert }) => {
        const result = yangZhangVolatility(data)
        assert.equal(result.volatility.length, data.length)
        assert.equal(result.overnightVol.length, data.length)
        assert.equal(result.intradayVol.length, data.length)
        assert.equal(result.rogersSatchell.length, data.length)
    })

    test('non-NaN volatility values are >= 0', ({ assert }) => {
        const result = yangZhangVolatility(data)
        for (const val of result.volatility) {
            if (!isNaN(val)) {
                assert.isTrue(
                    val >= 0,
                    `Expected volatility to be >= 0 but got ${val}`
                )
            }
        }
    })

    test('not enough data throws', ({ assert }) => {
        const tinyData = generateTestData(2)
        try {
            yangZhangVolatility(tinyData)
            assert.fail('Expected error was not thrown')
        } catch (error) {
            assert.isNotNull(error)
        }
    })
})
