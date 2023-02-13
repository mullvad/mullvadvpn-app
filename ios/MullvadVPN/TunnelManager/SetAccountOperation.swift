//
//  SetAccountOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 16/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import Operations
import class WireGuardKitTypes.PrivateKey
import class WireGuardKitTypes.PublicKey

enum SetAccountAction {
    /// Set new account.
    case new

    /// Set existing account.
    case existing(String)

    /// Unset account.
    case unset

    var taskName: String {
        switch self {
        case .new:
            return "Set new account"
        case .existing:
            return "Set existing account"
        case .unset:
            return "Unset account"
        }
    }
}

private struct SetAccountResult {
    let accountData: StoredAccountData
    let privateKey: PrivateKey
    let device: REST.Device
}

private struct SetAccountContext: OperationInputContext {
    var accountData: StoredAccountData?
    var privateKey: PrivateKey?
    var device: REST.Device?

    func reduce() -> SetAccountResult? {
        guard let accountData = accountData,
              let privateKey = privateKey,
              let device = device
        else {
            return nil
        }

        return SetAccountResult(
            accountData: accountData,
            privateKey: privateKey,
            device: device
        )
    }
}

class SetAccountOperation: ResultOperation<StoredAccountData?, Error> {
    private let interactor: TunnelInteractor
    private let accountsProxy: REST.AccountsProxy
    private let devicesProxy: REST.DevicesProxy
    private let action: SetAccountAction

    private let logger = Logger(label: "SetAccountOperation")
    private let operationQueue = AsyncOperationQueue()

    private var children: [Operation] = []

    init(
        dispatchQueue: DispatchQueue,
        interactor: TunnelInteractor,
        accountsProxy: REST.AccountsProxy,
        devicesProxy: REST.DevicesProxy,
        action: SetAccountAction
    ) {
        self.interactor = interactor
        self.accountsProxy = accountsProxy
        self.devicesProxy = devicesProxy
        self.action = action

        super.init(dispatchQueue: dispatchQueue)
    }

    override func main() {
        let deleteDeviceOperation = getDeleteDeviceOperation()
        let unsetDeviceStateOperation = getUnsetDeviceStateOperation()

        deleteDeviceOperation.flatMap { unsetDeviceStateOperation.addDependency($0) }

        let setupAccountOperations = getAccountDataOperation()
            .flatMap { accountOperation -> [Operation] in
                accountOperation.addCondition(
                    NoFailedDependenciesCondition(ignoreCancellations: false)
                )
                accountOperation.addDependency(unsetDeviceStateOperation)

                let createDeviceOperation = getCreateDeviceOperation()
                createDeviceOperation.addCondition(
                    NoFailedDependenciesCondition(ignoreCancellations: false)
                )
                createDeviceOperation.inject(from: accountOperation)

                let saveSettingsOperation = getSaveSettingsOperation()
                saveSettingsOperation.addCondition(
                    NoFailedDependenciesCondition(ignoreCancellations: false)
                )

                saveSettingsOperation.injectMany(context: SetAccountContext())
                    .inject(from: accountOperation, assignOutputTo: \.accountData)
                    .inject(from: createDeviceOperation, via: { context, output in
                        let (privateKey, device) = output

                        context.privateKey = privateKey
                        context.device = device
                    })
                    .reduce()

                saveSettingsOperation.addBlockObserver(
                    OperationBlockObserver(didFinish: { operation, error in
                        self.completeOperation(accountData: operation.output)
                    })
                )

                return [accountOperation, createDeviceOperation, saveSettingsOperation]
            } ?? []

        var enqueueOperations: [Operation] = [deleteDeviceOperation, unsetDeviceStateOperation]
            .compactMap { $0 }
        enqueueOperations.append(contentsOf: setupAccountOperations)

        if setupAccountOperations.isEmpty {
            let finishingOperation = BlockOperation()
            finishingOperation.completionBlock = { [weak self] in
                self?.completeOperation(accountData: nil)
            }
            finishingOperation.addDependencies(enqueueOperations)
            enqueueOperations.append(finishingOperation)
        }

        children = enqueueOperations
        operationQueue.addOperations(enqueueOperations, waitUntilFinished: false)
    }

    override func operationDidCancel() {
        operationQueue.cancelAllOperations()
    }

    // MARK: - Private

    private func completeOperation(accountData: StoredAccountData?) {
        guard !isCancelled else {
            finish(completion: .cancelled)
            return
        }

        let errors = children.compactMap { operation -> Error? in
            return (operation as? AsyncOperation)?.error
        }

        if let error = errors.first {
            finish(completion: .failure(error))
        } else {
            finish(completion: .success(accountData))
        }
    }

    private func getAccountDataOperation() -> ResultOperation<StoredAccountData, Error>? {
        switch action {
        case .new:
            return getCreateAccountOperation()

        case let .existing(accountNumber):
            return getExistingAccountOperation(accountNumber: accountNumber)

        case .unset:
            return nil
        }
    }

    private func getCreateAccountOperation() -> ResultBlockOperation<StoredAccountData, Error> {
        let operation = ResultBlockOperation<StoredAccountData, Error>(dispatchQueue: dispatchQueue)

        operation.setExecutionBlock { operation in
            self.logger.debug("Create new account...")

            let task = self.accountsProxy.createAccount(retryStrategy: .default) { completion in
                let mappedCompletion = completion.mapError { error -> Error in
                    self.logger.error(
                        error: error,
                        message: "Failed to create new account."
                    )
                    return error
                }.map { newAccountData -> StoredAccountData in
                    self.logger.debug("Created new account.")

                    return StoredAccountData(
                        identifier: newAccountData.id,
                        number: newAccountData.number,
                        expiry: newAccountData.expiry
                    )
                }

                operation.finish(completion: mappedCompletion)
            }

            operation.addCancellationBlock {
                task.cancel()
            }
        }

        return operation
    }

    private func getExistingAccountOperation(accountNumber: String)
        -> ResultOperation<StoredAccountData, Error>
    {
        let operation = ResultBlockOperation<StoredAccountData, Error>(dispatchQueue: dispatchQueue)

        operation.setExecutionBlock { operation in
            self.logger.debug("Request account data...")

            let task = self.accountsProxy.getAccountData(
                accountNumber: accountNumber,
                retryStrategy: .default
            ) { completion in
                let mappedCompletion = completion.mapError { error -> Error in
                    self.logger.error(
                        error: error,
                        message: "Failed to receive account data."
                    )
                    return error
                }.map { accountData -> StoredAccountData in
                    self.logger.debug("Received account data.")

                    return StoredAccountData(
                        identifier: accountData.id,
                        number: accountNumber,
                        expiry: accountData.expiry
                    )
                }

                operation.finish(completion: mappedCompletion)
            }

            operation.addCancellationBlock {
                task.cancel()
            }
        }

        return operation
    }

    private func getDeleteDeviceOperation() -> ResultBlockOperation<Void, Error>? {
        guard case let .loggedIn(accountData, deviceData) = interactor.deviceState else {
            return nil
        }

        let operation = ResultBlockOperation<Void, Error>(dispatchQueue: dispatchQueue)

        operation.setExecutionBlock { operation in
            self.logger.debug("Delete current device...")

            let task = self.devicesProxy.deleteDevice(
                accountNumber: accountData.number,
                identifier: deviceData.identifier,
                retryStrategy: .default
            ) { completion in
                let mappedCompletion = completion
                    .mapError { error -> Error in
                        self.logger.error(
                            error: error,
                            message: "Failed to delete device."
                        )
                        return error
                    }.map { isDeleted in
                        if isDeleted {
                            self.logger.debug("Deleted device.")
                        } else {
                            self.logger.debug("Device is already deleted.")
                        }
                    }

                operation.finish(completion: mappedCompletion)
            }

            operation.addCancellationBlock {
                task.cancel()
            }
        }

        return operation
    }

    private func getUnsetDeviceStateOperation() -> AsyncBlockOperation {
        return AsyncBlockOperation(dispatchQueue: dispatchQueue) { operation in
            // Tell the caller to unsubscribe from VPN status notifications.
            self.interactor.prepareForVPNConfigurationDeletion()

            // Reset tunnel and device state.
            self.interactor.updateTunnelStatus { tunnelStatus in
                tunnelStatus = TunnelStatus()
                tunnelStatus.state = .disconnected
            }
            self.interactor.setDeviceState(.loggedOut, persist: true)

            // Finish immediately if tunnel provider is not set.
            guard let tunnel = self.interactor.tunnel else {
                operation.finish()
                return
            }

            // Remove VPN configuration.
            tunnel.removeFromPreferences { error in
                self.dispatchQueue.async {
                    // Ignore error but log it.
                    if let error = error {
                        self.logger.error(
                            error: error,
                            message: "Failed to remove VPN configuration."
                        )
                    }

                    self.interactor.setTunnel(nil, shouldRefreshTunnelState: false)

                    operation.finish()
                }
            }
        }
    }

    private func getCreateDeviceOperation()
        -> TransformOperation<StoredAccountData, (PrivateKey, REST.Device), Error>
    {
        let createDeviceOperation = TransformOperation<
            StoredAccountData,
            (PrivateKey, REST.Device),
            Error
        >(dispatchQueue: dispatchQueue)

        createDeviceOperation.setExecutionBlock { storedAccountData, operation in
            self.logger.debug("Store last used account.")

            do {
                try SettingsManager.setLastUsedAccount(storedAccountData.number)
            } catch {
                self.logger.error(
                    error: error,
                    message: "Failed to store last used account number."
                )
            }

            self.logger.debug("Create device...")

            let privateKey = PrivateKey()

            let request = REST.CreateDeviceRequest(
                publicKey: privateKey.publicKey,
                hijackDNS: false
            )

            let task = self.devicesProxy.createDevice(
                accountNumber: storedAccountData.number,
                request: request,
                retryStrategy: .default
            ) { completion in
                let mappedCompletion = completion
                    .map { device in
                        return (privateKey, device)
                    }
                    .mapError { error -> Error in
                        self.logger.error(error: error, message: "Failed to create device.")
                        return error
                    }

                operation.finish(completion: mappedCompletion)
            }

            operation.addCancellationBlock {
                task.cancel()
            }
        }

        return createDeviceOperation
    }

    private func getSaveSettingsOperation()
        -> TransformOperation<SetAccountResult, StoredAccountData, Error>
    {
        let saveSettingsOperation = TransformOperation<
            SetAccountResult, StoredAccountData, Error
        >(dispatchQueue: dispatchQueue)

        saveSettingsOperation.setExecutionBlock { input in
            self.logger.debug("Saving settings...")

            let device = input.device
            let newDeviceState = DeviceState.loggedIn(
                input.accountData,
                StoredDeviceData(
                    creationDate: device.created,
                    identifier: device.id,
                    name: device.name,
                    hijackDNS: device.hijackDNS,
                    ipv4Address: device.ipv4Address,
                    ipv6Address: device.ipv6Address,
                    wgKeyData: StoredWgKeyData(
                        creationDate: Date(),
                        privateKey: input.privateKey
                    )
                )
            )

            self.interactor.setSettings(TunnelSettingsV2(), persist: true)
            self.interactor.setDeviceState(newDeviceState, persist: true)

            return input.accountData
        }

        return saveSettingsOperation
    }
}
