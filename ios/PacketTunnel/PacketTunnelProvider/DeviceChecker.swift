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

final class DeviceChecker {
    private let dispatchQueue = DispatchQueue(label: "DeviceCheckerQueue")
    private let operationQueue = AsyncOperationQueue.makeSerial()

    private let accountsProxy: RESTAccountHandling
    private let devicesProxy: DeviceHandling

    init(accountsProxy: RESTAccountHandling, devicesProxy: DeviceHandling) {
        self.accountsProxy = accountsProxy
        self.devicesProxy = devicesProxy
    }

    /**
     Start device diagnostics to determine the reason why the tunnel is not functional.

     This involves the following steps:

     1. Fetch account and device data.
     2. Check account validity and whether it has enough time left.
     3. Verify that current device is registered with backend and that both device and backend point to the same public
        key.
     4. Rotate WireGuard key on key mismatch.
     */
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
