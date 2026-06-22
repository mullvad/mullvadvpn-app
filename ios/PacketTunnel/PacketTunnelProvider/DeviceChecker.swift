//
//  DeviceChecker.swift
//  PacketTunnel
//
//  Created by pronebird on 12/09/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadSettings
import MullvadTypes
import Operations
import PacketTunnelCore

final class DeviceChecker {
    private let dispatchQueue = DispatchQueue(label: "DeviceCheckerQueue")
    private let operationQueue = AsyncOperationQueue.makeSerial()

    private let accountsProxy: RESTAccountHandling
    private let devicesProxy: DeviceHandling
    private let settingsManager: SettingsManager

    init(accountsProxy: RESTAccountHandling, devicesProxy: DeviceHandling, settingsManager: SettingsManager) {
        self.accountsProxy = accountsProxy
        self.devicesProxy = devicesProxy
        self.settingsManager = settingsManager
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
    func start(rotateKeyOnMismatch: Bool) async -> Result<DeviceCheck, Error> {
        let checkOperation = DeviceCheckOperation(
            dispatchQueue: dispatchQueue,
            remoteSevice: DeviceCheckRemoteService(accountsProxy: accountsProxy, devicesProxy: devicesProxy),
            deviceStateAccessor: DeviceStateAccessor(settingsManager: settingsManager),
            rotateImmediatelyOnKeyMismatch: rotateKeyOnMismatch
        )

        return await withTaskCancellationHandler {
            return await withCheckedContinuation { continuation in
                checkOperation.completionHandler = { result in
                    continuation.resume(with: .success(result))
                }
                operationQueue.addOperation(checkOperation)
            }
        } onCancel: {
            checkOperation.cancel()
        }
    }
}
