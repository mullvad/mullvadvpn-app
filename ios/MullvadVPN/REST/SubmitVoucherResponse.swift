//
//  SubmitVoucherResponse.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-08-24.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension REST {
    struct SubmitVoucherResponse: Codable {
        let timeAdded: Int
        let newExpiry: String
    }
}
