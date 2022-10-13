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

class UpdateAccountDataOperation: ResultOperation<Void, Error> {
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
            finish(completion: .failure(InvalidDeviceStateError()))
            return
        }

        task = accountsProxy.getAccountData(
            accountNumber: accountData.number,
            retryStrategy: .default
        ) { completion in
            self.dispatchQueue.async {
                self.didReceiveAccountData(completion: completion)
            }
        }
    }

    override func operationDidCancel() {
        task?.cancel()
        task = nil
    }

    private func didReceiveAccountData(
        completion: OperationCompletion<REST.AccountData, REST.Error>
    ) {
        let mappedCompletion = completion.mapError { error -> Error in
            self.logger.error(
                error: error,
                message: "Failed to fetch account expiry."
            )
            return error
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

        finish(completion: mappedCompletion)
    }
}
