//
//  UpdateDeviceDataOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 13/05/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging
import class WireGuardKitTypes.PublicKey

class UpdateDeviceDataOperation: ResultOperation<StoredDeviceData, TunnelManager.Error> {
    private let logger = Logger(label: "UpdateDeviceDataOperation")

    private let state: TunnelManager.State
    private let devicesProxy: REST.DevicesProxy

    private var task: Cancellable?

    init(
        dispatchQueue: DispatchQueue,
        state: TunnelManager.State,
        devicesProxy: REST.DevicesProxy
    )
    {
        self.state = state
        self.devicesProxy = devicesProxy

        super.init(dispatchQueue: dispatchQueue)
    }

    override func main() {
        guard let tunnelSettings = state.tunnelSettings else {
            finish(completion: .failure(.unsetAccount))
            return
        }

        task = devicesProxy.getDevice(
            accountNumber: tunnelSettings.account.number,
            identifier: tunnelSettings.device.identifier,
            retryStrategy: .default,
            completion: { [weak self] completion in
                self?.dispatchQueue.async {
                    self?.didReceiveDeviceResponse(
                        tunnelSettings: tunnelSettings,
                        completion: completion
                    )
                }
            })
    }

    override func operationDidCancel() {
        task?.cancel()
        task = nil
    }

    private func didReceiveDeviceResponse(
        tunnelSettings: TunnelSettingsV2,
        completion: OperationCompletion<REST.Device?, REST.Error>
    ) {
        let mappedCompletion = completion
            .mapError { error -> TunnelManager.Error in
                return .getDevice(error)
            }
            .flatMap { device -> OperationCompletion<REST.Device, TunnelManager.Error> in
                if let device = device {
                    return .success(device)
                } else {
                    return .failure(.deviceRevoked)
                }
            }

        guard let device = mappedCompletion.value else {
            finish(completion: mappedCompletion.assertNoSuccess())
            return
        }

        do {
            var newTunnelSettings = tunnelSettings
            newTunnelSettings.device.update(from: device)

            try SettingsManager.writeSettings(newTunnelSettings)

            finish(completion: .success(newTunnelSettings.device))
        } catch {
            logger.error(
                chainedError: AnyChainedError(error),
                message: "Failed to write settings."
            )

            finish(completion: .failure(.writeSettings(error)))
        }
    }

}
