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
        guard count > 0 else { return [] }
        guard length > 0 && length < count else { return [suffix(from: startIndex)] }

        return (0 ... ((count - 1) / length))
            .map { dropFirst($0 * length).prefix(length) }
    }
}
