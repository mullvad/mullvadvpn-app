//
//  RedeemVoucherInteractor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-08-30.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

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
        tasks.append(tunnelManager.redeemVoucher(code) { [weak self] result in
            guard let self else { return }
            completion(result)
            guard shouldVerifyVoucherAsAccount,
                  result.error?.isInvalidVoucher ?? false else {
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
        let task = accountsProxy.getAccountData(
            accountNumber: code,
            retryStrategy: .noRetry
        ) { [weak self] result in
            guard let self,
                  case .success = result else {
                return
            }
            showLogoutDialog?()
            preferredAccountNumber = code
        }

        tasks.append(task)
    }
}

fileprivate extension Error {
    var isInvalidVoucher: Bool {
        (self as? REST.Error)?.compareErrorCode(.invalidVoucher) ?? false
    }
}
