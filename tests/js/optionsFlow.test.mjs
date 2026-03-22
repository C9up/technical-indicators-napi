import { test } from '@japa/runner'
import pkg from '../../index.js'
const { optionsFlowScore } = pkg

/**
 * Build a minimal set of synthetic option contracts around a given spot price.
 * @param {number} spot
 * @param {number} count
 * @returns {Array<{strike: number, openInterest: number, volume: number, dte: number, side: string, impliedVolatility: number}>}
 */
function buildContracts(spot, count = 20) {
    const contracts = []
    for (let i = 0; i < count; i++) {
        const offset = (i - count / 2) * (spot * 0.01)
        contracts.push({
            strike: parseFloat((spot + offset).toFixed(2)),
            openInterest: Math.floor(500 + Math.random() * 4500),
            volume: Math.floor(100 + Math.random() * 900),
            dte: Math.floor(1 + Math.random() * 90),
            side: i % 2 === 0 ? 'call' : 'put',
            impliedVolatility: parseFloat((0.15 + Math.random() * 0.6).toFixed(4)),
        })
    }
    return contracts
}

test.group('Options Flow Score', (group) => {

    const spotPrice = 150
    let contracts

    group.setup(() => {
        contracts = buildContracts(spotPrice, 30)
    })

    test('results are sorted by score descending', ({ assert }) => {
        const result = optionsFlowScore(contracts, spotPrice)
        for (let i = 1; i < result.length; i++) {
            assert.isAtLeast(
                result[i - 1].score,
                result[i].score,
                `result[${i - 1}].score should be >= result[${i}].score`
            )
        }
    })

    test('returns at most topN results', ({ assert }) => {
        const topN = 5
        const result = optionsFlowScore(contracts, spotPrice, topN)
        assert.isAtMost(result.length, topN)
    })

    test('all scores are finite numbers', ({ assert }) => {
        const result = optionsFlowScore(contracts, spotPrice)
        for (const item of result) {
            assert.isTrue(Number.isFinite(item.score), `score ${item.score} should be finite`)
        }
    })

    test('result items contain index and strike fields', ({ assert }) => {
        const result = optionsFlowScore(contracts, spotPrice)
        for (const item of result) {
            assert.property(item, 'index')
            assert.property(item, 'strike')
            assert.property(item, 'score')
        }
    })

    test('result index values reference valid contract positions', ({ assert }) => {
        const result = optionsFlowScore(contracts, spotPrice)
        for (const item of result) {
            assert.isAtLeast(item.index, 0)
            assert.isBelow(item.index, contracts.length)
        }
    })

    test('result strike matches the contract at the reported index', ({ assert }) => {
        const result = optionsFlowScore(contracts, spotPrice)
        for (const item of result) {
            assert.approximately(item.strike, contracts[item.index].strike, 0.0001)
        }
    })

    test('empty contracts throws', ({ assert }) => {
        try {
            optionsFlowScore([], spotPrice)
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.instanceOf(error, Error)
        }
    })

    test('spotPrice of zero throws', ({ assert }) => {
        try {
            optionsFlowScore(contracts, 0)
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.instanceOf(error, Error)
        }
    })

    test('negative spotPrice throws', ({ assert }) => {
        try {
            optionsFlowScore(contracts, -50)
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.instanceOf(error, Error)
        }
    })

    test('topN equal to 1 returns a single result', ({ assert }) => {
        const result = optionsFlowScore(contracts, spotPrice, 1)
        assert.isAtMost(result.length, 1)
    })

    test('topN larger than contracts count returns all contracts', ({ assert }) => {
        const smallContracts = buildContracts(spotPrice, 5)
        const result = optionsFlowScore(smallContracts, spotPrice, 1000)
        assert.isAtMost(result.length, smallContracts.length)
    })

    test('all scores are numbers (not NaN)', ({ assert }) => {
        const result = optionsFlowScore(contracts, spotPrice)
        for (const item of result) {
            assert.isFalse(isNaN(item.score), `score should not be NaN`)
        }
    })
})
