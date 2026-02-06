//
//  RevokedDeviceViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-10-02.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadMockData
import SwiftUI

class RevokedDeviceViewModel: ObservableObject {
    private var interactor: RevokedDeviceInteractorProtocol
    var tunnelState: TunnelState

    var onLogout: (() -> Void)?

    init(interactor: RevokedDeviceInteractorProtocol) {
        self.interactor = interactor
        self.tunnelState = interactor.tunnelStatus.state

        self.interactor.didUpdateTunnelStatus = { [weak self] status in
            self?.tunnelState = status.state
        }
    }
}
