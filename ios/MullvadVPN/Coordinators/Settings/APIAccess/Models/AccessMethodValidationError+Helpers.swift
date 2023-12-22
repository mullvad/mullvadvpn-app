//
//  AccessMethodValidationError+Helpers.swift
//  MullvadVPN
//
//  Created by pronebird on 29/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

extension AccessMethodValidationError {
    /// Checks if any of the fields associated with the given access method have validation errors.
    ///
    /// - Parameter selectedMethod: the access method specified in view model
    /// - Returns: `true` if any of the fields associated with the given access method have validation errors, otherwise false.
    func containsProxyConfigurationErrors(selectedMethod: AccessMethodKind) -> Bool {
        switch selectedMethod {
        case .direct, .bridges:
            false
        case .shadowsocks:
            fieldErrors.contains { $0.context == .shadowsocks }
        case .socks5:
            fieldErrors.contains { $0.context == .socks }
        }
    }
}
