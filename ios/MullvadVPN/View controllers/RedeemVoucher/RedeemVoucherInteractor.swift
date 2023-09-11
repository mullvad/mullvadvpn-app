//
//  RedeemVoucherInteractor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-08-30.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

final class RedeemVoucherInteractor {
    private let tunnelManager: TunnelManager
    private let accountsProxy: REST.AccountsProxy
    private let shouldVerifyVoucherAsAccount: Bool
    
    private var tasks: [Cancellable] = []
    private var preferredAccountNumber: String?

    var showLogoutDialog: (() -> Void)?
    var didLogout: (() -> Void)?
    
    init(
        tunnelManager: TunnelManager,
        accountsProxy: REST.AccountsProxy,
        verifyVoucherAsAccount: Bool
    ) {
        self.tunnelManager = tunnelManager
        self.accountsProxy = accountsProxy
        self.shouldVerifyVoucherAsAccount = verifyVoucherAsAccount
    }

    func redeemVoucher(
        code: String,
        completion: @escaping ((Result<REST.SubmitVoucherResponse, Error>) -> Void)
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

    func logout(completionHandler: @escaping () -> Void) {
        preferredAccountNumber.flatMap { accountNumber in
            tunnelManager.unsetAccount { [weak self] in
                guard let self else {
                    return
                }
                completionHandler()
                didLogout?()
                notify(accountNumber: accountNumber)
            }
        }
    }

    func cancelAll() {
        tasks.forEach { $0.cancel() }
    }

    private func verifyVoucherAsAccount(code: String) {
        let executer = accountsProxy.getAccountData(accountNumber: code)
        tasks.append(executer.execute(retryStrategy: .noRetry) { [weak self] result in
            guard let self,
                  case .success = result else {
                return
            }
            showLogoutDialog?()
            preferredAccountNumber = code
        })
    }

    /**
     Name of notification posted when current account number changes.
     */
    static let didChangePreferredAccountNumber = Notification
        .Name(rawValue: "CreateAccountVoucherCoordinatorDidChangeAccountNumber")

    /**
     User info key passed along with `didChangePreferredAccountNumber` notification that contains string value that
     indicates the new account number.
     */
    static let preferredAccountNumberUserInfoKey = "preferredAccountNumber"

    /// Posts `didChangePreferredAccountNumber` notification.
    private func notify(accountNumber: String) {
        NotificationCenter.default.post(
            name: Self.didChangePreferredAccountNumber,
            object: self,
            userInfo: [Self.preferredAccountNumberUserInfoKey: accountNumber]
        )
    }
}

fileprivate extension Error {
    var isInvalidVoucher: Bool {
        (self as? REST.Error)?.compareErrorCode(.invalidVoucher) ?? false
    }
}
