import { test } from '@japa/runner'
import { extractImportantLevels } from '../../index.js'

/**
 * extractImportantLevels(data: number[]): ImportantLevels
 *
 * Returns:
 *   highestResistance: number  — max of resistances union all data
 *   lowestSupport:     number  — min of supports union all data
 *   averagePivot:      number  — mean of detected pivot points (or mean of data if none)
 *   supports:          number[]
 *   resistances:       number[]
 *
 * Detection uses a sliding window of WINDOW=5: a point is a local max (resistance)
 * if it is >= all 5 neighbours on each side; a local min (support) if <= all 5 neighbours.
 */

/** Generate a simple sine-wave price series with a known range */
function buildSineData(count, amplitude = 20, base = 100) {
    return Array.from({ length: count }, (_, i) => base + amplitude * Math.sin(i * (2 * Math.PI / 20)))
}

test.group('ExtractImportantLevels', (group) => {

    test('returns an object with all required fields', ({ assert }) => {
        const data = buildSineData(50)
        const result = extractImportantLevels(data)
        assert.property(result, 'highestResistance')
        assert.property(result, 'lowestSupport')
        assert.property(result, 'averagePivot')
        assert.property(result, 'supports')
        assert.property(result, 'resistances')
    })

    test('supports and resistances are arrays', ({ assert }) => {
        const data = buildSineData(50)
        const result = extractImportantLevels(data)
        assert.isArray(result.supports)
        assert.isArray(result.resistances)
    })

    test('highestResistance is >= every value in the input data', ({ assert }) => {
        const data = buildSineData(60)
        const result = extractImportantLevels(data)
        const max = Math.max(...data)
        assert.isAtLeast(result.highestResistance, max - 0.0001)
    })

    test('lowestSupport is <= every value in the input data', ({ assert }) => {
        const data = buildSineData(60)
        const result = extractImportantLevels(data)
        const min = Math.min(...data)
        assert.isAtMost(result.lowestSupport, min + 0.0001)
    })

    test('highestResistance >= lowestSupport', ({ assert }) => {
        const data = buildSineData(60)
        const result = extractImportantLevels(data)
        assert.isAtLeast(result.highestResistance, result.lowestSupport)
    })

    test('averagePivot is a finite number', ({ assert }) => {
        const data = buildSineData(50)
        const result = extractImportantLevels(data)
        assert.isNumber(result.averagePivot)
        assert.isTrue(isFinite(result.averagePivot))
    })

    test('averagePivot lies between lowestSupport and highestResistance', ({ assert }) => {
        const data = buildSineData(80)
        const result = extractImportantLevels(data)
        assert.isAtLeast(result.averagePivot, result.lowestSupport - 0.0001)
        assert.isAtMost(result.averagePivot, result.highestResistance + 0.0001)
    })

    test('sine wave of 60 points produces at least one support and one resistance', ({ assert }) => {
        // A full sine wave has clear local maxima and minima beyond the 5-point window
        const data = buildSineData(60)
        const result = extractImportantLevels(data)
        assert.isAbove(result.supports.length, 0, 'should detect at least one support')
        assert.isAbove(result.resistances.length, 0, 'should detect at least one resistance')
    })

    test('all support values are <= highestResistance and >= lowestSupport', ({ assert }) => {
        const data = buildSineData(60)
        const result = extractImportantLevels(data)
        result.supports.forEach((s, i) => {
            assert.isAtMost(s, result.highestResistance + 0.0001, `supports[${i}] must be <= highestResistance`)
            assert.isAtLeast(s, result.lowestSupport - 0.0001, `supports[${i}] must be >= lowestSupport`)
        })
    })

    test('all resistance values are <= highestResistance and >= lowestSupport', ({ assert }) => {
        const data = buildSineData(60)
        const result = extractImportantLevels(data)
        result.resistances.forEach((r, i) => {
            assert.isAtMost(r, result.highestResistance + 0.0001, `resistances[${i}] must be <= highestResistance`)
            assert.isAtLeast(r, result.lowestSupport - 0.0001, `resistances[${i}] must be >= lowestSupport`)
        })
    })

    test('monotone increasing data produces no detected local supports or resistances', ({ assert }) => {
        // Strictly monotone has no local extrema so both arrays should be empty
        const data = Array.from({ length: 30 }, (_, i) => 100 + i)
        const result = extractImportantLevels(data)
        assert.lengthOf(result.supports, 0)
        assert.lengthOf(result.resistances, 0)
    })

    test('monotone increasing: averagePivot falls back to mean of input data', ({ assert }) => {
        const data = Array.from({ length: 30 }, (_, i) => 100 + i)
        const mean = data.reduce((acc, v) => acc + v, 0) / data.length
        const result = extractImportantLevels(data)
        assert.approximately(result.averagePivot, mean, 0.001)
    })

    test('highestResistance equals max of data for monotone series', ({ assert }) => {
        const data = Array.from({ length: 20 }, (_, i) => 50 + i * 2)
        const result = extractImportantLevels(data)
        const max = Math.max(...data)
        assert.approximately(result.highestResistance, max, 0.001)
    })

    test('lowestSupport equals min of data for monotone series', ({ assert }) => {
        const data = Array.from({ length: 20 }, (_, i) => 50 + i * 2)
        const result = extractImportantLevels(data)
        const min = Math.min(...data)
        assert.approximately(result.lowestSupport, min, 0.001)
    })

    test('single repeated value: highestResistance and lowestSupport equal that value', ({ assert }) => {
        const data = Array.from({ length: 20 }, () => 75.5)
        const result = extractImportantLevels(data)
        assert.approximately(result.highestResistance, 75.5, 0.001)
        assert.approximately(result.lowestSupport, 75.5, 0.001)
    })
})
