//
//  CreateAccountVoucherInteractor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-08-30.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

final class CreateAccountVoucherInteractor: RedeemVoucherProtocol {
    private let tunnelManager: TunnelManager
    private let accountsProxy: REST.AccountsProxy
    var didInputAccountNumber: ((String) -> Void)?
    var tasks: [Cancellable] = []

    init(
        tunnelManager: TunnelManager,
        accountsProxy: REST.AccountsProxy
    ) {
        self.tunnelManager = tunnelManager
        self.accountsProxy = accountsProxy
    }

    func redeemVoucher(
        code: String,
        completion: @escaping ((Result<REST.SubmitVoucherResponse, Error>) -> Void)
    ) {
        tasks.append(tunnelManager.redeemVoucher(code) { result in
            switch result {
            case let .success(value):
                completion(.success(value))
            case let .failure(error):
                completion(.failure(error))
                guard error.isInvalidVoucher else {
                    return
                }
                self.getAccount(number: code)
            }
        })
    }

    func logout(_ completion: @escaping () -> Void) {
        tunnelManager.unsetAccount(completionHandler: completion)
    }

    private func getAccount(number: String) {
        let executer = accountsProxy.getAccountData(accountNumber: number)
        tasks.append(executer.execute(retryStrategy: .noRetry) { result in
            guard case .success = result else {
                return
            }
            self.didInputAccountNumber?(number)
        })
    }
}

fileprivate extension Error {
    var isInvalidVoucher: Bool {
        (self as? REST.Error)?.compareErrorCode(.invalidVoucher) ?? false
    }
}
