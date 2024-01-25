//
//  LocalNetworkProbe.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2024-01-25.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

struct LocalNetworkProbe {
    /// Does a best effort attempt to trigger the local network privacy alert.
    func triggerLocalNetworkPrivacyAlert() {
        let dispatchQueue = DispatchQueue(label: "com.mullvad.localNetworkAlert")
        let localIpv4Connection = NWConnection(
            to: NWEndpoint.hostPort(host: .ipv4(.broadcast), port: .any),
            using: .udp
        )
        localIpv4Connection.start(queue: dispatchQueue)

        let localIpv6Connection = NWConnection(
            to: NWEndpoint.hostPort(host: .ipv6(.broadcast), port: .any),
            using: .udp
        )
        localIpv6Connection.start(queue: dispatchQueue)
    }
}
