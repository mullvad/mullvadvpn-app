//
//  RevokedDeviceInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol RevokedDeviceInteractorProtocol {
    var didUpdateTunnelStatus: ((TunnelStatus) -> Void)? { get set }
    var tunnelStatus: TunnelStatus { get }
}

final class RevokedDeviceInteractor: RevokedDeviceInteractorProtocol {
    private let tunnelManager: TunnelManager
    private var tunnelObserver: TunnelObserver?

    var didUpdateTunnelStatus: ((TunnelStatus) -> Void)?

    var tunnelStatus: TunnelStatus {
        tunnelManager.tunnelStatus
    }

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager

        let tunnelObserver =
            TunnelBlockObserver(didUpdateTunnelStatus: { [weak self] _, tunnelStatus in
                self?.didUpdateTunnelStatus?(tunnelStatus)
            })

        tunnelManager.addObserver(tunnelObserver)

        self.tunnelObserver = tunnelObserver
    }
}

class MockRevokedDeviceInteractor: RevokedDeviceInteractorProtocol {
    var tunnelStatus: TunnelStatus
    var didUpdateTunnelStatus: ((TunnelStatus) -> Void)?

    init(tunnelStatus: TunnelStatus) {
        self.tunnelStatus = tunnelStatus
    }
}
