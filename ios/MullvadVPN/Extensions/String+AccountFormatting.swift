//
//  String+AccountFormatting.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-06-10.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension String {
    var formattedAccountNumber: String {
        return split(every: 4).joined(separator: " ")
    }
}
