//
//  TunnelInfo.swift
//  TunnelInfo
//
//  Created by pronebird on 10/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Struct that holds current account token and tunnel settings.
struct TunnelInfo: Equatable {
    /// Mullvad account token
    var token: String

    /// Tunnel settings
    var tunnelSettings: TunnelSettings
}
