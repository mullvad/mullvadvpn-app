// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadREST
import MullvadTypes

final class RedeemVoucherInteractor: @unchecked Sendable {
    private let tunnelManager: TunnelManager
    private let accountsProxy: RESTAccountHandling
    private let shouldVerifyVoucherAsAccount: Bool

    private var tasks: [Cancellable] = []
    private var preferredAccountNumber: String?

    var showLogoutDialog: (() -> Void)?
    var didLogout: ((String) -> Void)?

    init(
        tunnelManager: TunnelManager,
        accountsProxy: RESTAccountHandling,
        verifyVoucherAsAccount: Bool
    ) {
        self.tunnelManager = tunnelManager
        self.accountsProxy = accountsProxy
        self.shouldVerifyVoucherAsAccount = verifyVoucherAsAccount
    }

    func redeemVoucher(
        code: String,
        completion: @escaping (@Sendable (Result<REST.SubmitVoucherResponse, Error>) -> Void)
    ) {
        tasks.append(
            tunnelManager.redeemVoucher(code) { [weak self] result in
                guard let self else { return }
                completion(result)
                guard shouldVerifyVoucherAsAccount,
                    result.error?.isInvalidVoucher ?? false
                else {
                    return
                }
                verifyVoucherAsAccount(code: code)
            })
    }

    func logout() async {
        guard let accountNumber = preferredAccountNumber else { return }
        await tunnelManager.unsetAccount()
        didLogout?(accountNumber)
    }

    func cancelAll() {
        tasks.forEach { $0.cancel() }
    }

    private func verifyVoucherAsAccount(code: String) {
        let task = Task {
            let result = await accountsProxy.getAccountData(
                accountNumber: code,
                retryStrategy: .noRetry
            )

            if case .success = result {
                showLogoutDialog?()
                preferredAccountNumber = code
            }
        }

        tasks.append(task.cancellable)
    }
}

fileprivate extension Error {
    var isInvalidVoucher: Bool {
        (self as? REST.Error)?.compareErrorCode(.invalidVoucher) ?? false
    }
}
