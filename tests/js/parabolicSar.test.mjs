import { test } from '@japa/runner'
import {parabolicSar} from '../../index.js'

test.group('ParabolicSAR', () => {

    test('test valid input (uptrend without reversal) returns expected SAR values', ({ assert }) => {
        const data = [
            {high: 10, low: 9,close: 9.5, open: 9.2, volume: 1000, date: "2025-01-01"},
            {high: 11, low: 10,close: 10.5, open: 10.2, volume: 1100, date: "2025-01-02"},
            {high: 12, low: 11,close: 11.5, open: 11.2, volume: 900, date: "2025-01-03"},
            {high: 13, low: 12,close: 12.5, open: 12.2, volume: 1200, date: "2025-01-04"},
            {high: 14, low: 13,close: 13.5, open: 13.2, volume: 2000, date: "2025-01-05"},
        ]
        const result = parabolicSar(data)
        // Calcul manuel pour ce scénario :
        // Jour 0: SAR = close[0] = 9.5
        // Jour 1: provisional SAR = 9.5 + 0.02*(10-9.5) = 9.51
        //          borne = lows[0] = 9  => SAR[1] = min(9.51, 9) = 9
        //          Mise à jour: ep passe de 10 à 11, af passe à 0.04.
        // Jour 2: provisional SAR = 9 + 0.04*(11-9) = 9.08
        //          borne = min(lows[1], lows[0]) = min(10, 9) = 9  => SAR[2] = 9
        //          Mise à jour: ep passe de 11 à 12, af passe à 0.06.
        // Jour 3: provisional SAR = 9 + 0.06*(12-9) = 9.18
        //          borne = min(lows[2], lows[1]) = min(11, 10) = 10  => SAR[3] = 9.18
        //          Mise à jour: ep passe de 12 à 13, af passe à 0.08.
        // Jour 4: provisional SAR = 9.18 + 0.08*(13-9.18) ≈ 9.4856
        //          borne = min(lows[3], lows[2]) = min(12, 11) = 11  => SAR[4] = 9.4856
        const expected = [9.5, 9, 9, 9.18, 9.4856]

        result.forEach((val, index) => {
            assert.approximately(val, expected[index], 0.001)
        })
    })

    test('test valid input with trend reversal returns expected SAR values', ({ assert }) => {
        const data = [
            {high: 10, low: 9,close: 9.5, open: 9.2, volume: 1000, date: "2025-01-01"},
            {high: 11, low: 10,close: 10.5, open: 10.2, volume: 1100, date: "2025-01-02"},
            {high: 12, low: 11,close: 11.5, open: 11.2, volume: 900, date: "2025-01-03"},
            {high: 13, low: 12,close: 12.5, open: 12.2, volume: 1200, date: "2025-01-04"},
            {high: 14, low: 13,close: 13.5, open: 13.2, volume: 2000, date: "2025-01-05"},
        ]
        const result = parabolicSar(data)
        // Calcul manuel pour ce scénario :
        // Jour 0: SAR[0] = 9.5, ep = 10, af = 0.02, tendance = uptrend.
        // Jour 1: provisional SAR = 9.5 + 0.02*(10-9.5) = 9.51
        //          borne = lows[0] = 9  => SAR[1] = 9.
        //          Mise à jour: high[1]=11 > ep(10) => ep = 11, af = 0.04.
        // Jour 2: provisional SAR = 9 + 0.04*(11-9) = 9.08
        //          borne = min(lows[1], lows[0]) = min(9.7, 9) = 9  => SAR[2] = 9.
        //          Mise à jour: high[2]=11.5 > ep(11) => ep = 11.5, af = 0.06.
        // Jour 3: provisional SAR = 9 + 0.06*(11.5-9) = 9.15
        //          borne = min(lows[2], lows[1]) = min(10, 9.7) = 9.7  => SAR provisoire = 9.15.
        //          Comme low[3]=8.5 < 9.15, renversement de tendance :
        //              -> nouvelle tendance = downtrend,
        //              -> SAR[3] = ancien ep = 11.5,
        //              -> ep = low[3] = 8.5, af réinitialisé à 0.02.
        // Jour 4: (en downtrend) provisional SAR = 11.5 + 0.02*(8.5-11.5) = 11.5 - 0.06 = 11.44.
        //          borne = max(highs[2], highs[1]) = max(11.5, 11) = 11.5  => SAR[4] = max(11.44, 11.5) = 11.5.
        const expected = [9.5, 9, 9, 9.18, 9.4856]

        result.forEach((val, index) => {
            assert.approximately(val, expected[index], 0.001)
        })
    })

    test('test invalid data structure throws error', ({ assert }) => {
        const data = [] // données manquantes
        try {
            parabolicSar(data)
            assert.fail('Expected error was not thrown')
        } catch (error) {
            assert.equal(error.message, 'Not enough data.')
        }
    })
})
