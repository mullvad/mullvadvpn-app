//
//  String+Search.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-04-02.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum SearchScore: Comparable {
    case none
    case score(Int)

    static func < (lhs: SearchScore, rhs: SearchScore) -> Bool {
        switch (lhs, rhs) {
        case (.none, .none): return false
        case (.none, _): return true
        case (_, .none): return false
        case (.score(let l), .score(let r)): return l < r
        }
    }
}

extension String {
    func search(_ query: String) -> SearchScore {
        guard !query.isEmpty else { return .none }

        let text = self.localizedLowercase
        let query = query.trimmingCharacters(in: .whitespacesAndNewlines).localizedLowercase
        let textLength = text.count

        // Substring match
        if let range = text.range(of: query) {
            let index = text.distance(from: text.startIndex, to: range.lowerBound)

            // Starts with
            if index == 0 {
                return .score(1000)
            }

            var currentIndex = 0
            let words = text.split(separator: " ")
            for word in words {
                if word.hasPrefix(query) {
                    // earlier word = higher score
                    let normalizedPosition = Double(currentIndex) / Double(textLength)
                    return .score(Int(800 * (1 - normalizedPosition)))
                }
                currentIndex += word.count + 1
            }

            // Contains match
            return .score(500 - index)
        } else if fuzzyMatch(text, query: query) {
            return .score(300)
        }
        return .none
    }

    private func fuzzyMatch(_ text: String, query: String) -> Bool {
        var tIndex = text.startIndex

        for char in query {
            if let found = text[tIndex...].firstIndex(of: char) {
                tIndex = text.index(after: found)
            } else {
                return false
            }
        }

        return true
    }
}
