//
//  UpdateAccountDataTask.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 31/07/2026.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadREST
import MullvadSettings
import MullvadTypes

actor UpdateAccountDataTask {
    private let logger = Logger(label: "UpdateAccountDataTask")
    private let interactor: TunnelInteractor
    private let accountsProxy: RESTAccountHandling
    private var task: Cancellable?

    init(
        interactor: TunnelInteractor,
        accountsProxy: RESTAccountHandling
    ) {
        self.interactor = interactor
        self.accountsProxy = accountsProxy
    }

    func start() async -> Result<Void, Error> {
        guard case let .loggedIn(accountData, _) = interactor.deviceState else {
            return .failure(InvalidDeviceStateError())
        }

        let result: Result<Account, Error> = await withTaskCancellationHandler {
            await withCheckedContinuation { continuation in
                task = accountsProxy.getAccountData(
                    accountNumber: accountData.number,
                    retryStrategy: .default,
                    completion: { result in
                        continuation.resume(returning: result)
                    }
                )
            }
        } onCancel: {
            Task { await cancel() }
        }

        return didReceiveAccountData(result: result)
    }

    func cancel() {
        task?.cancel()
        task = nil
    }

    private func didReceiveAccountData(result: Result<Account, Error>) -> Result<Void, Error> {
        let result = result.inspectError { error in
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

        return result
    }
}
