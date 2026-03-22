import { test } from '@japa/runner'
import pkg from '../../index.js'
const { gaussianMixture } = pkg

test.group('GaussianMixture', (group) => {

    const nComponents = 3
    const nFeatures = 2

    // 60 numbers = 30 data points, each with 2 features
    const makeData = (n = 30) => {
        const data = []
        for (let i = 0; i < n * nFeatures; i++) {
            data.push(Math.sin(i * 0.3) * 10 + 50 + (i % 7) * 2)
        }
        return data
    }

    test('labels length equals data.length / nFeatures', ({ assert }) => {
        const data = makeData(30)
        const result = gaussianMixture(data, nFeatures, nComponents)
        assert.equal(result.labels.length, data.length / nFeatures)
    })

    test('all labels are between 0 and nComponents - 1', ({ assert }) => {
        const data = makeData(30)
        const result = gaussianMixture(data, nFeatures, nComponents)
        for (const label of result.labels) {
            assert.isTrue(label >= 0 && label <= nComponents - 1,
                `Label ${label} is out of range [0, ${nComponents - 1}]`)
        }
    })

    test('clusters length equals nComponents', ({ assert }) => {
        const data = makeData(30)
        const result = gaussianMixture(data, nFeatures, nComponents)
        assert.equal(result.clusters.length, nComponents)
    })

    test('each cluster has mean, variance, and weight', ({ assert }) => {
        const data = makeData(30)
        const result = gaussianMixture(data, nFeatures, nComponents)
        for (const cluster of result.clusters) {
            assert.property(cluster, 'mean')
            assert.property(cluster, 'variance')
            assert.property(cluster, 'weight')
        }
    })

    test('sum of cluster weights is approximately 1', ({ assert }) => {
        const data = makeData(30)
        const result = gaussianMixture(data, nFeatures, nComponents)
        const weightSum = result.clusters.reduce((acc, c) => acc + c.weight, 0)
        assert.approximately(weightSum, 1.0, 0.01)
    })

    test('BIC is a finite number', ({ assert }) => {
        const data = makeData(30)
        const result = gaussianMixture(data, nFeatures, nComponents)
        assert.isTrue(Number.isFinite(result.bic),
            `Expected BIC to be finite, got ${result.bic}`)
    })

    test('probabilities length equals number of data points times nComponents', ({ assert }) => {
        const data = makeData(30)
        const result = gaussianMixture(data, nFeatures, nComponents)
        assert.equal(result.probabilities.length, (data.length / nFeatures) * nComponents)
    })

    test('logLikelihood is a finite number', ({ assert }) => {
        const data = makeData(30)
        const result = gaussianMixture(data, nFeatures, nComponents)
        assert.isTrue(Number.isFinite(result.logLikelihood),
            `Expected logLikelihood to be finite, got ${result.logLikelihood}`)
    })

    test('iterations is a positive integer', ({ assert }) => {
        const data = makeData(30)
        const result = gaussianMixture(data, nFeatures, nComponents)
        assert.isTrue(Number.isInteger(result.iterations) && result.iterations > 0,
            `Expected iterations to be a positive integer, got ${result.iterations}`)
    })

    test('same seed produces reproducible results', ({ assert }) => {
        const data = makeData(30)
        const seed = 42
        const result1 = gaussianMixture(data, nFeatures, nComponents, 100, 1e-6, true, seed)
        const result2 = gaussianMixture(data, nFeatures, nComponents, 100, 1e-6, true, seed)
        assert.deepEqual(result1.labels, result2.labels)
        assert.approximately(result1.bic, result2.bic, 1e-10)
    })

    test('data length not divisible by nFeatures throws', ({ assert }) => {
        // 7 values with nFeatures=2 is not divisible
        const data = [1, 2, 3, 4, 5, 6, 7]
        try {
            gaussianMixture(data, nFeatures, nComponents)
            assert.fail('Expected an error to be thrown')
        } catch (error) {
            assert.isTrue(error instanceof Error)
        }
    })

})
