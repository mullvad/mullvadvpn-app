//
//  UInt+Counting.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension UInt {
    /// Determines whether a number has a specific order in a given set.
    /// Eg. `6.isOrdered(nth: 3, forEverySetOf: 4)` -> "Is a 6 ordered third in an arbitrary
    /// amount of sets of four?". The result of this is `true`, since in a range of eg. 0-7 a six
    /// would be considered third if the range was divided into sets of 4.
    public func isOrdered(nth: UInt, forEverySetOf set: UInt) -> Bool {
        guard nth > 0, set > 0 else {
            assertionFailure("Both 'nth' and 'set' must be positive")
            return false
        }

        return self % set == nth - 1
    }
}
