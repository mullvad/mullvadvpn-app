//
//  PhraseCoder.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2026-07-31.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

struct PhraseCoder {
    let digits: Int
    let alphabet: [String]
    let codespace: UInt64

    init(digits: Int, words: [String]) {
        self.digits = digits
        self.alphabet =
            words.filter({
                $0.count <= 3
            })
            + words.filter({
                $0.count > 3
            })
        self.codespace = UInt64(truncating: NSDecimalNumber(decimal: pow(10, digits)))
    }

    private func encode_number_to_indices(number: UInt64) -> [Int] {
        // convert to indices
        var result = [Int]()
        var input = number
        var remaining = codespace
        let alpha_size = UInt64(alphabet.count)
        while remaining > 0 {
            let symbol = Int(input % alpha_size)
            input /= alpha_size
            remaining /= alpha_size
            result.append(symbol)
        }
        return result
    }

    func encode_number(number: UInt64) -> String {
        encode_number_to_indices(number: number).map { alphabet[$0] }.joined(separator: " ")
    }

    func encode(input: String) -> String? {
        UInt64(input).map(self.encode_number)
    }

    private func wordIndex(_ word: any StringProtocol) -> Int? {
        alphabet.enumerated().first { $0.element == word }.map { $0.offset }
    }

    private func decodeToNumber(input: String) -> UInt64? {
        // TODO: reject if any words are not found
        let codes = input.split(separator: " ").compactMap { wordIndex($0) }.reversed()
        let alpha_size = UInt64(alphabet.count)
        return codes.reduce(UInt64(0)) { s, i in
            s * alpha_size + UInt64(i)
        }
    }

    func decode(input: String) -> String? {
        decodeToNumber(input: input).map(String.init)
    }
}
