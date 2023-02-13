//
//  UpdateDeviceDataOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 13/05/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadTypes
import Operations
import class WireGuardKitTypes.PublicKey

class UpdateDeviceDataOperation: ResultOperation<StoredDeviceData> {
    private let interactor: TunnelInteractor
    private let devicesProxy: REST.DevicesProxy

    private var task: Cancellable?

    init(
        dispatchQueue: DispatchQueue,
        interactor: TunnelInteractor,
        devicesProxy: REST.DevicesProxy
    ) {
        self.interactor = interactor
        self.devicesProxy = devicesProxy

        super.init(dispatchQueue: dispatchQueue)
    }

    override func main() {
        guard case let .loggedIn(accountData, deviceData) = interactor.deviceState else {
            finish(result: .failure(InvalidDeviceStateError()))
            return
        }

        task = devicesProxy.getDevice(
            accountNumber: accountData.number,
            identifier: deviceData.identifier,
            retryStrategy: .default,
            completion: { [weak self] result in
                self?.dispatchQueue.async {
                    self?.didReceiveDeviceResponse(result: result)
                }
            }
        )
    }

    override func operationDidCancel() {
        task?.cancel()
        task = nil
    }

    private func didReceiveDeviceResponse(result: Result<REST.Device, Error>) {
        let result = result.tryMap { device -> StoredDeviceData in
            switch interactor.deviceState {
            case .loggedIn(let storedAccount, var storedDevice):
                storedDevice.update(from: device)
                let newDeviceState = DeviceState.loggedIn(storedAccount, storedDevice)
                interactor.setDeviceState(newDeviceState, persist: true)

                return storedDevice

            default:
                throw InvalidDeviceStateError()
            }
        }

        finish(result: result)
    }
}
