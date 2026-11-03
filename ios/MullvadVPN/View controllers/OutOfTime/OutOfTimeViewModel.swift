// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import SwiftUI

protocol OutOfTimeViewModelProtocol {
    var isConnected: Bool { get }
    var disconnectButtonIsEnabled: Bool { get }
    var showInAppPurchase: ((PaymentAction) -> Void)? { get }
    var didAddMoreCredit: (() -> Void)? { get }
    func disconnect()
    func startAccountPolling()
    func stopAccountPolling()
}

@Observable final class OutOfTimeViewModel: OutOfTimeViewModelProtocol {
    private let interactor: OutOfTimeInteractor

    var isConnected: Bool = false
    var disconnectButtonIsEnabled: Bool = true

    var showInAppPurchase: ((PaymentAction) -> Void)?
    var didAddMoreCredit: (() -> Void)?

    init(interactor: OutOfTimeInteractor) {
        self.interactor = interactor
        isConnected = interactor.tunnelStatus.state.isSecured

        interactor.didReceiveTunnelStatus { [weak self] in
            self?.isConnected = $0.state.isSecured
        }

        interactor.didAddMoreCredit = { [weak self] in
            self?.didAddMoreCredit?()
        }
    }

    func disconnect() {
        disconnectButtonIsEnabled = false
        interactor.stopTunnel()
    }

    func startAccountPolling() {
        interactor.startAccountUpdateTimer()
    }

    func stopAccountPolling() {
        interactor.startAccountUpdateTimer()
    }
}

final class OutOfTimeViewModelMock: OutOfTimeViewModelProtocol {
    let isConnected: Bool
    let disconnectButtonIsEnabled: Bool

    var showInAppPurchase: ((PaymentAction) -> Void)?
    var didAddMoreCredit: (() -> Void)?

    init(isConnected: Bool, disconnectButtonIsEnabled: Bool) {
        self.isConnected = isConnected
        self.disconnectButtonIsEnabled = disconnectButtonIsEnabled
    }

    func disconnect() {}
    func startAccountPolling() {}
    func stopAccountPolling() {}
}
