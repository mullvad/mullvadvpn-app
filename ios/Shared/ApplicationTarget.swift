//
//  ApplicationTarget.swift
//  MullvadVPN
//
//  Created by pronebird on 09/06/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum ApplicationTarget: CaseIterable {
    case mainApp, packetTunnel

    /// Returns target bundle identifier.
    var bundleIdentifier: String {
        let key = "MainApplicationIdentifier"
        // swiftlint:disable:next force_cast
        let mainBundleIdentifier = Bundle.main.object(forInfoDictionaryKey: key) as! String
        switch self {
        case .mainApp:
            return mainBundleIdentifier
        case .packetTunnel:
            return "\(mainBundleIdentifier).PacketTunnel"
        }
    }
}
