//
//  RotateKeyOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 15/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging
import class WireGuardKitTypes.PrivateKey

class RotateKeyOperation: ResultOperation<TunnelManager.KeyRotationResult, TunnelManager.Error> {
    private let state: TunnelManager.State

    private let devicesProxy: REST.DevicesProxy
    private var task: Cancellable?

    private let rotationInterval: TimeInterval?
    private let logger = Logger(label: "ReplaceKeyOperation")

    init(
        dispatchQueue: DispatchQueue,
        state: TunnelManager.State,
        devicesProxy: REST.DevicesProxy,
        rotationInterval: TimeInterval?,
        completionHandler: @escaping CompletionHandler
    ) {
        self.state = state

        self.devicesProxy = devicesProxy
        self.rotationInterval = rotationInterval

        super.init(
            dispatchQueue: dispatchQueue,
            completionQueue: dispatchQueue,
            completionHandler: completionHandler
        )
    }

    override func main() {
        guard let tunnelSettings = state.tunnelSettings else {
            finish(completion: .failure(.unsetAccount))
            return
        }

        if let rotationInterval = rotationInterval {
            let creationDate = tunnelSettings.device.wgKeyData.creationDate
            let nextRotationDate = creationDate.addingTimeInterval(rotationInterval)

            if nextRotationDate > Date() {
                logger.debug("Throttle private key rotation.")

                finish(completion: .success(.throttled(creationDate)))
                return
            } else {
                logger.debug("Private key is old enough, rotate right away.")
            }
        } else {
            logger.debug("Rotate private key right away.")
        }

        logger.debug("Replacing old key with new key on server...")

        let newPrivateKey = PrivateKey()

        task = devicesProxy.rotateDeviceKey(
            accountNumber: tunnelSettings.account.number,
            identifier: tunnelSettings.device.identifier,
            publicKey: newPrivateKey.publicKey,
            retryStrategy: .default
        ) { completion in
            self.dispatchQueue.async {
                self.didRotateKey(
                    tunnelSettings: tunnelSettings,
                    newPrivateKey: newPrivateKey,
                    completion: completion
                )
            }
        }
    }

    override func operationDidCancel() {
        task?.cancel()
        task = nil
    }

    private func didRotateKey(
        tunnelSettings: TunnelSettingsV2,
        newPrivateKey: PrivateKey,
        completion: OperationCompletion<REST.Device, REST.Error>
    )
    {
        let mappedCompletion = completion.mapError { error -> TunnelManager.Error in
            logger.error(
                chainedError: error,
                message: "Failed to rotate device key."
            )

            return .rotateKey(error)
        }

        guard let device = mappedCompletion.value else {
            finish(completion: mappedCompletion.assertNoSuccess())
            return
        }

        logger.debug("Successfully rotated device key. Persisting settings...")

        do {
            var newTunnelSettings = tunnelSettings
            newTunnelSettings.device.update(from: device)
            newTunnelSettings.device.wgKeyData = StoredWgKeyData(
                creationDate: Date(),
                privateKey: newPrivateKey
            )

            try SettingsManager.writeSettings(newTunnelSettings)

            state.tunnelSettings = newTunnelSettings

            finish(completion: .success(.finished))
        } catch {
            logger.error(
                chainedError: AnyChainedError(error),
                message: "Failed to write settings."
            )

            finish(completion: .failure(.writeSettings(error)))
        }
    }
}
