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
import MullvadSettings
import MullvadTypes
import Operations

protocol AccountManagerTunnelInteractor: Sendable {
    var deviceState: DeviceState { get }
    func setDeviceState(_ deviceState: DeviceState, persist: Bool)
    func setLastUsedAccount(_ accountNumber: String)
    func unsetTunnelConfiguration()
    func removeLastUsedAccount()
    func handleRestError(_ error: Error)
}

struct AccountManager: Sendable {
    let operationQueue: AsyncOperationQueue
    let internalQueue: DispatchQueue
    let interactor: TunnelInteractor
    let backgroundTaskProvider: BackgroundTaskProviding
    let accountsProxy: RESTAccountHandling
    let devicesProxy: DeviceHandling
    let category: String

    func setAccount(
        action: SetAccountAction,
        completionHandler: @escaping @Sendable (Result<StoredAccountData?, Error>) -> Void
    ) {
        Task {
            let deviceState = await interactor.getDeviceState()

            let operation = SetAccountOperation(
                dispatchQueue: internalQueue,
                accountsProxy: accountsProxy,
                devicesProxy: devicesProxy,
                action: action,
                deviceState: { deviceState },
                onUpdateAccount: { deviceState, completion in
                    Task {
                        if let accountNumber = deviceState.accountData?.number {
                            await interactor.setLastUsedAccount(accountNumber)
                        }
                        await interactor.setDeviceState(deviceState, persist: true)
                        completion?()
                    }
                }
            )

            operation.completionQueue = .main
            operation.completionHandler = { result in
                completionHandler(result)
            }

            operation.addObserver(
                BackgroundObserver(
                    backgroundTaskProvider: backgroundTaskProvider,
                    name: action.taskName,
                    cancelUponExpiration: true
                ))

            operation.addCondition(MutuallyExclusive(category: category))

            // Unsetting (ie. logging out) or deleting the account should cancel all other
            // currently ongoing activity.
            switch action {
            case .unset, .delete:
                operationQueue.cancelAllOperations()
            default:
                break
            }

            operationQueue.addOperation(operation)
        }
    }

    func rotatePrivateKey(completionHandler: @escaping @Sendable (Result<Void, Error>) -> Void) async -> Cancellable {
        let deviceState = await interactor.getDeviceState()

        let operation = RotateKeyOperation(dispatchQueue: internalQueue, devicesProxy: devicesProxy) {
            deviceState
        } onUpdateAccount: { deviceState in
            Task { await interactor.setDeviceState(deviceState, persist: true) }
        }

        operation.completionQueue = .main
        operation.completionHandler = { result in
            completionHandler(result)
        }

        operation.addObserver(
            BackgroundObserver(
                backgroundTaskProvider: backgroundTaskProvider,
                name: "Rotate private key",
                cancelUponExpiration: true
            )
        )

        operation.addCondition(
            MutuallyExclusive(category: category)
        )

        operationQueue.addOperation(operation)

        return operation
    }

    func updateAccountData() async throws {
        guard case let .loggedIn(accountData, _) = await interactor.getDeviceState() else {
            throw InvalidDeviceStateError()
        }

        let result = await accountsProxy.getAccountData(
            accountNumber: accountData.number,
            retryStrategy: .default
        )

        do {
            let accountData = try result.get()
            switch await interactor.getDeviceState() {
            case .loggedIn(var storedAccountData, let storedDeviceData):
                storedAccountData.expiry = accountData.expiry
                let newDeviceState = DeviceState.loggedIn(storedAccountData, storedDeviceData)

                // Make sure we don't update any data if cancellation happened in-flight.
                if Task.isCancelled {
                    throw CancellationError()
                } else {
                    await interactor.setDeviceState(newDeviceState, persist: true)
                }
            default:
                throw InvalidDeviceStateError()
            }
        } catch {
            await interactor.handleRestError(error)
            throw error
        }
    }

    func updateDeviceData() async throws {
        guard case let .loggedIn(accountData, deviceData) = await interactor.getDeviceState() else {
            throw InvalidDeviceStateError()
        }
        do {
            let device = try await devicesProxy.getDevice(
                accountNumber: accountData.number,
                identifier: deviceData.identifier,
                retryStrategy: .default
            )
            switch await interactor.getDeviceState() {
            case .loggedIn(let storedAccount, var storedDevice):
                storedDevice.update(from: device)
                let newDeviceState = DeviceState.loggedIn(storedAccount, storedDevice)
                await interactor.setDeviceState(newDeviceState, persist: true)
            default:
                throw InvalidDeviceStateError()
            }
        } catch {
            await interactor.handleRestError(error)
            throw error
        }
    }
}
