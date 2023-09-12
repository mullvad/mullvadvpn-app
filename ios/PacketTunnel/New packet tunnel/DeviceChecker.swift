//
//  DeviceChecker.swift
//  PacketTunnel
//
//  Created by pronebird on 12/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes
import Operations
import PacketTunnelCore

class DeviceChecker: DeviceCheckProtocol {
    private let dispatchQueue = DispatchQueue(label: "DeviceCheckerQueue")
    private let operationQueue = AsyncOperationQueue.makeSerial()

    private let accountsProxy: REST.AccountsProxy
    private let devicesProxy: REST.DevicesProxy

    init(accountsProxy: REST.AccountsProxy, devicesProxy: REST.DevicesProxy) {
        self.accountsProxy = accountsProxy
        self.devicesProxy = devicesProxy
    }

    func start(rotateKeyOnMismatch: Bool) async throws -> DeviceCheck {
        let checkOperation = DeviceCheckOperation(
            dispatchQueue: dispatchQueue,
            remoteSevice: DeviceCheckRemoteService(accountsProxy: accountsProxy, devicesProxy: devicesProxy),
            deviceStateAccessor: DeviceStateAccessor(),
            rotateImmediatelyOnKeyMismatch: rotateKeyOnMismatch
        )

        return try await withTaskCancellationHandler {
            return try await withCheckedThrowingContinuation { continuation in
                checkOperation.completionHandler = { result in
                    continuation.resume(with: result)
                }
                operationQueue.addOperation(checkOperation)
            }
        } onCancel: {
            checkOperation.cancel()
        }
    }
}
