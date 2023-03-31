//
//  String+FuzzyMatch.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-04-02.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension String {
    func fuzzyMatch(_ needle: String) -> Bool {
        guard !needle.isEmpty else { return false }

        let haystack = lowercased()
        let needle = needle.lowercased()

        var indices: [Index] = []
        var remainder = needle[...].utf8

        for index in haystack.utf8.indices {
            let character = haystack.utf8[index]

            if character == remainder[remainder.startIndex] {
                indices.append(index)
                remainder.removeFirst()

                if remainder.isEmpty {
                    return !indices.isEmpty
                }
            }
        }

        return false
    }
}
