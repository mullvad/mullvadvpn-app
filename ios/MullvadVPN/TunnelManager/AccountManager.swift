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
}

struct AccountManager: Sendable {
    let operationQueue: AsyncOperationQueue
    let internalQueue: DispatchQueue
    let interactor: AccountManagerTunnelInteractor
    let backgroundTaskProvider: BackgroundTaskProviding
    let accountsProxy: RESTAccountHandling
    let devicesProxy: DeviceHandling
    let category: String

    func setAccount(
        action: SetAccountAction,
        completionHandler: @escaping @Sendable (Result<StoredAccountData?, Error>) -> Void
    ) {
        let operation = SetAccountOperation(
            dispatchQueue: internalQueue,
            accountsProxy: accountsProxy,
            devicesProxy: devicesProxy,
            action: action,
            deviceState: {
                interactor.deviceState
            },
            onUpdateAccount: { deviceState in
                interactor.setDeviceState(deviceState, persist: true)
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

    func rotatePrivateKey(completionHandler: @escaping @Sendable (Result<Void, Error>) -> Void) -> Cancellable {
        let operation = RotateKeyOperation(dispatchQueue: internalQueue, devicesProxy: devicesProxy) {
            interactor.deviceState
        } onUpdateAccount: { deviceState in
            interactor.setDeviceState(deviceState, persist: true)
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

    func updateDeviceData(_ completionHandler: (@Sendable (Error?) -> Void)? = nil) -> Cancellable {
        let operation = UpdateDeviceDataOperation(
            dispatchQueue: internalQueue,
            devicesProxy: devicesProxy,
            deviceState: {
                interactor.deviceState
            },
            onUpdateAccount: { deviceState in
                interactor.setDeviceState(deviceState, persist: true)
            }
        )

        operation.completionQueue = .main
        operation.completionHandler = { completion in
            completionHandler?(completion.error)
        }

        operation.addObserver(
            BackgroundObserver(
                backgroundTaskProvider: backgroundTaskProvider,
                name: "Update device data",
                cancelUponExpiration: true
            )
        )

        operation.addCondition(MutuallyExclusive(category: category))
        operationQueue.addOperation(operation)
        return operation
    }
}
