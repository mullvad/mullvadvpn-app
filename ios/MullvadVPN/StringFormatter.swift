//
//  StringFormatter.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-06-10.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

struct StringFormatter {
    static func formattedAccountNumber(from string: String) -> String {
        return string.split(every: 4).joined(separator: " ")
    }
}
