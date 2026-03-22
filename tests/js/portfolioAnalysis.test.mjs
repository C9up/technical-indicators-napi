import { test } from '@japa/runner'
import pkg from '../../index.js'
const { covarianceMatrix, portfolioStats, efficientFrontier } = pkg

/**
 * Build a flat daily returns array for `nAssets` assets over `nDays` days.
 * Layout: [asset0_day0, asset1_day0, ..., asset0_day1, asset1_day1, ...]
 * i.e. row-major with nAssets columns.
 */
function makeReturnsFlat(nAssets, nDays) {
    const returns = []
    let seed = 7
    for (let d = 0; d < nDays; d++) {
        for (let a = 0; a < nAssets; a++) {
            seed = (seed * 1664525 + 1013904223) & 0xffffffff
            returns.push(((seed >>> 0) / 0xffffffff) * 0.06 - 0.02)
        }
    }
    return returns
}

test.group('PortfolioAnalysis', (group) => {

    const nAssets = 4
    const nDays = 252
    const returnsFlat = makeReturnsFlat(nAssets, nDays)
    const equalWeights = Array.from({ length: nAssets }, () => 1 / nAssets)

    // --- covarianceMatrix ---

    test('correlation diagonal equals 1.0', ({ assert }) => {
        const result = covarianceMatrix(returnsFlat, nAssets)
        for (let i = 0; i < nAssets; i++) {
            const diagIndex = i * nAssets + i
            assert.approximately(result.correlation[diagIndex], 1.0, 1e-9,
                `Diagonal element [${i},${i}] should be 1.0`)
        }
    })

    test('all correlations are between -1 and 1', ({ assert }) => {
        const result = covarianceMatrix(returnsFlat, nAssets)
        for (let i = 0; i < result.correlation.length; i++) {
            const c = result.correlation[i]
            assert.isTrue(c >= -1.0 - 1e-9 && c <= 1.0 + 1e-9,
                `correlation[${i}] = ${c} is outside [-1, 1]`)
        }
    })

    test('covarianceMatrix result has expected fields', ({ assert }) => {
        const result = covarianceMatrix(returnsFlat, nAssets)
        assert.properties(result, ['covariance', 'correlation', 'meanReturns', 'volatilities', 'nAssets'])
    })

    test('covarianceMatrix nAssets matches input', ({ assert }) => {
        const result = covarianceMatrix(returnsFlat, nAssets)
        assert.equal(result.nAssets, nAssets)
    })

    test('covariance matrix is symmetric', ({ assert }) => {
        const result = covarianceMatrix(returnsFlat, nAssets)
        for (let i = 0; i < nAssets; i++) {
            for (let j = 0; j < nAssets; j++) {
                const ij = result.covariance[i * nAssets + j]
                const ji = result.covariance[j * nAssets + i]
                assert.approximately(ij, ji, 1e-12,
                    `Covariance[${i},${j}]=${ij} != Covariance[${j},${i}]=${ji}`)
            }
        }
    })

    // --- portfolioStats ---

    test('portfolioStats with equal weights returns valid stats', ({ assert }) => {
        const result = portfolioStats(returnsFlat, nAssets, equalWeights)
        assert.properties(result, ['expectedReturnDaily', 'volatilityDaily', 'sharpeRatio'])
        assert.isTrue(Number.isFinite(result.expectedReturnDaily),
            'expectedReturnDaily should be finite')
        assert.isTrue(result.volatilityDaily >= 0,
            'volatilityDaily should be non-negative')
    })

    test('portfolioStats volatilityDaily is non-negative', ({ assert }) => {
        const result = portfolioStats(returnsFlat, nAssets, equalWeights)
        assert.isTrue(result.volatilityDaily >= 0)
    })

    // --- efficientFrontier ---

    test('efficientFrontier has nPoints entries', ({ assert }) => {
        const nPoints = 20
        const result = efficientFrontier(returnsFlat, nAssets, nPoints)
        assert.equal(result.frontier.length, nPoints)
    })

    test('efficientFrontier default nPoints produces non-empty frontier', ({ assert }) => {
        const result = efficientFrontier(returnsFlat, nAssets)
        assert.isTrue(result.frontier.length > 0, 'Frontier should not be empty')
    })

    test('GMVP has the lowest volatility on the frontier', ({ assert }) => {
        const result = efficientFrontier(returnsFlat, nAssets, 20)
        const minFrontierVol = Math.min(...result.frontier.map(p => p.volatility))
        assert.approximately(result.gmvp.volatility, minFrontierVol, 1e-6,
            `GMVP volatility ${result.gmvp.volatility} is not the minimum on the frontier ${minFrontierVol}`)
    })

    test('maxSharpe.sharpeRatio is >= gmvp.sharpeRatio', ({ assert }) => {
        const result = efficientFrontier(returnsFlat, nAssets, 20)
        assert.isTrue(result.maxSharpe.sharpeRatio >= result.gmvp.sharpeRatio - 1e-9,
            `maxSharpe Sharpe ${result.maxSharpe.sharpeRatio} < gmvp Sharpe ${result.gmvp.sharpeRatio}`)
    })

    test('efficientFrontier result has gmvp and maxSharpe fields', ({ assert }) => {
        const result = efficientFrontier(returnsFlat, nAssets, 10)
        assert.property(result, 'gmvp')
        assert.property(result, 'maxSharpe')
        assert.property(result, 'frontier')
    })

    test('less than 2 assets throws', ({ assert }) => {
        const singleAssetReturns = makeReturnsFlat(1, 100)
        try {
            covarianceMatrix(singleAssetReturns, 1)
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.isTrue(error instanceof Error)
        }
    })

})
