//
//  IteratorProtocol+String.swift
//  MullvadVPN
//
//  Created by pronebird on 03/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension IteratorProtocol where Element == Character {
    /// Collect characters into a string while the predicate evaluates to `true`. Rethrows errors thrown by the predicate.
    ///
    /// - Parameter predicate: The predicate to evaluate each character before appending it to the result string.
    /// - Returns: The result string.
    mutating func take(while predicate: (Character) throws -> Bool) rethrows -> String {
        var accummulated = ""

        while let char = next() {
            guard try predicate(char) else { break }

            accummulated.append(char)
        }

        return accummulated
    }
}
