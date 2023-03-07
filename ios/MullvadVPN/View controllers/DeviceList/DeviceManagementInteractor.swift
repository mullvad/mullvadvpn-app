//
//  DeviceManagementInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 26/07/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes
import Operations

class DeviceManagementInteractor {
    private let devicesProxy: REST.DevicesProxy
    private let accountNumber: String

    init(accountNumber: String, devicesProxy: REST.DevicesProxy) {
        self.accountNumber = accountNumber
        self.devicesProxy = devicesProxy
    }

    @discardableResult
    func getDevices(
        _ completionHandler: @escaping (Result<[REST.Device], Error>)
            -> Void
    ) -> Cancellable {
        return devicesProxy.getDevices(
            accountNumber: accountNumber,
            retryStrategy: .default,
            completion: completionHandler
        )
    }

    @discardableResult
    func deleteDevice(
        _ identifier: String,
        completionHandler: @escaping (Result<Bool, Error>) -> Void
    ) -> Cancellable {
        return devicesProxy.deleteDevice(
            accountNumber: accountNumber,
            identifier: identifier,
            retryStrategy: .default,
            completion: completionHandler
        )
    }
}
