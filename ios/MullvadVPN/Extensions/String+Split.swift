//
//  String+Split.swift
//  MullvadVPN
//
//  Created by pronebird on 27/03/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension String {
    /// Returns the array of the longest possible subsequences of the given length.
    func split(every length: Int) -> [Substring] {
        guard length > 0 else { return [prefix(upTo: endIndex)] }

        let resultCount = Int((Float(count) / Float(length)).rounded(.up))

        return (0 ..< resultCount)
            .map { dropFirst($0 * length).prefix(length) }
    }
}
