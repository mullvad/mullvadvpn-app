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

class UpdateDeviceDataOperation: ResultOperation<StoredDeviceData, Error> {
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
            finish(completion: .failure(InvalidDeviceStateError()))
            return
        }

        task = devicesProxy.getDevice(
            accountNumber: accountData.number,
            identifier: deviceData.identifier,
            retryStrategy: .default,
            completion: { [weak self] completion in
                self?.dispatchQueue.async {
                    self?.didReceiveDeviceResponse(
                        completion: completion
                    )
                }
            }
        )
    }

    override func operationDidCancel() {
        task?.cancel()
        task = nil
    }

    private func didReceiveDeviceResponse(completion: OperationCompletion<REST.Device, REST.Error>)
    {
        let mappedCompletion = completion
            .tryMap { device -> StoredDeviceData in
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

        finish(completion: mappedCompletion)
    }
}
