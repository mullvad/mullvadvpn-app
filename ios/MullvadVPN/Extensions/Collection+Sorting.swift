//
//  Collection+Sorting.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-14.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension Collection where Element: StringProtocol {
    public func caseInsensitiveSorted() -> [Element] {
        sorted { $0.caseInsensitiveCompare($1) == .orderedAscending }
    }
}

extension MutableCollection where Element: StringProtocol, Self: RandomAccessCollection {
    public mutating func caseInsensitiveSort() {
        sort { $0.caseInsensitiveCompare($1) == .orderedAscending }
    }
}
