//
//  UpdateAccountDataOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 12/05/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadTypes
import Operations

class UpdateAccountDataOperation: ResultOperation<Void> {
    private let logger = Logger(label: "UpdateAccountDataOperation")
    private let interactor: TunnelInteractor
    private let accountsProxy: REST.AccountsProxy
    private var task: Cancellable?

    init(
        dispatchQueue: DispatchQueue,
        interactor: TunnelInteractor,
        accountsProxy: REST.AccountsProxy
    ) {
        self.interactor = interactor
        self.accountsProxy = accountsProxy

        super.init(dispatchQueue: dispatchQueue)
    }

    override func main() {
        guard case let .loggedIn(accountData, _) = interactor.deviceState else {
            finish(result: .failure(InvalidDeviceStateError()))
            return
        }

        task = accountsProxy.getAccountData(
            accountNumber: accountData.number,
            retryStrategy: .default
        ) { result in
            self.dispatchQueue.async {
                self.didReceiveAccountData(result: result)
            }
        }
    }

    override func operationDidCancel() {
        task?.cancel()
        task = nil
    }

    private func didReceiveAccountData(result: Result<REST.AccountData, Error>) {
        let result = result.inspectError { error in
            guard !error.isOperationCancellationError else { return }

            self.logger.error(
                error: error,
                message: "Failed to fetch account expiry."
            )
        }.tryMap { accountData in
            switch interactor.deviceState {
            case .loggedIn(var storedAccountData, let storedDeviceData):
                storedAccountData.expiry = accountData.expiry

                let newDeviceState = DeviceState.loggedIn(storedAccountData, storedDeviceData)

                interactor.setDeviceState(newDeviceState, persist: true)

            default:
                throw InvalidDeviceStateError()
            }
        }

        finish(result: result)
    }
}
