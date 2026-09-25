// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadTypes
import SwiftUI

@MainActor
protocol AccountViewModelProtocol {
    var deviceName: String { get }
    var accountNumber: String { get }
    var isOutOfTime: Bool { get }
    var paidUntil: Date { get }
    var onFinish: ((AccountDismissReason) -> Void)? { get }
    func deleteAccount() async throws
    func logout() async
    func addTime()
    func restorePurchase()
    func factoryReset() async
    func finishUnfinishedPurchase() async
    #if NEVER_IN_PRODUCTION
        func invalidateWireguardKey()
        func toggleGotaTun()
    #endif
}

extension AccountViewModelProtocol {
    var paidUntilString: String {
        DateFormatter.localizedString(
            from: paidUntil,
            dateStyle: .medium,
            timeStyle: .short
        )
    }
}

@Observable
class AccountViewModel: AccountViewModelProtocol {
    var deviceName: String {
        tunnelManager.deviceState.deviceData?.capitalizedName ?? ""
    }

    var accountNumber: String {
        tunnelManager.deviceState.accountData?.number ?? ""
    }

    var isOutOfTime: Bool {
        tunnelManager.deviceState.accountData?.isExpired ?? false
    }

    var paidUntil: Date {
        tunnelManager.deviceState.accountData?.expiry ?? Date()
    }

    let onFinish: ((AccountDismissReason) -> Void)?

    let onPaymentAction: (PaymentAction) -> Void

    let tunnelManager: TunnelManager

    init(
        tunnelManager: TunnelManager,
        onFinish: @escaping (AccountDismissReason) -> Void,
        onPaymentAction: @escaping (PaymentAction) -> Void
    ) {
        self.tunnelManager = tunnelManager
        self.onFinish = onFinish
        self.onPaymentAction = onPaymentAction
    }

    func deleteAccount() async throws {
        try await tunnelManager.deleteLoggedInAccount()
    }

    func logout() async {
        await tunnelManager.unsetAccount()
    }

    func addTime() {
        onPaymentAction(.purchase)
    }

    func restorePurchase() {
        onPaymentAction(.restorePurchase)
    }

    func factoryReset() async {
        tunnelManager.updateSettings([.reset])
        UserDefaults.standard.removePersistentDomain(forName: Bundle.main.bundleIdentifier!)
        await logout()
    }

    func finishUnfinishedPurchase() async {
        await StorePaymentManager.cleanupUnfinishedTransactions()
    }

    #if NEVER_IN_PRODUCTION
        func invalidateWireguardKey() {
            tunnelManager.invalidateWireGuardKey()
        }

        func toggleGotaTun() {
            PacketTunnelDebugSettings.useGotaTun = !PacketTunnelDebugSettings.useGotaTun
            tunnelManager.reapplyTunnelConfiguration()
        }
    #endif
}

@Observable
class AccountViewModelMock: AccountViewModelProtocol {
    let mockDeviceManaging = MockDeviceManaging()
    var currentDeviceId: String? {
        mockDeviceManaging.currentDeviceId
    }

    let accountNumber = "1234123412341234"
    let isOutOfTime = true
    let deviceName = "Easy Puma"
    let paidUntil = Date()
    let onFinish: ((AccountDismissReason) -> Void)? = {
        print("Finished, reason: \($0)")
    }

    func deleteAccount() async throws {}

    func logout() {}

    func addTime() {}

    func restorePurchase() {}

    func invalidateWireguardKey() {}

    func factoryReset() async {}

    func finishUnfinishedPurchase() async {}

    func toggleGotaTun() {}

}
