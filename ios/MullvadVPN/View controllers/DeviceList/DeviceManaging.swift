//
//  DeviceManagementInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 26/07/2022.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes
import WireGuardKitTypes

protocol DeviceManaging {
    var currentDeviceId: String? { get }
    func getDevices(_ completionHandler: @escaping @Sendable (Result<[Device], Error>) -> Void) -> Cancellable
    func deleteDevice(
        _ identifier: String,
        completionHandler: @escaping @Sendable (Result<Bool, Error>) -> Void
    ) -> Cancellable
}

class DeviceManagementInteractor: DeviceManaging, @unchecked Sendable {
    private let devicesProxy: DeviceHandling
    private let accountNumber: String
    let currentDeviceId: String?

    init(accountNumber: String, currentDeviceId: String? = nil, devicesProxy: DeviceHandling) {
        self.accountNumber = accountNumber
        self.devicesProxy = devicesProxy
        self.currentDeviceId = currentDeviceId
    }

    @discardableResult
    func getDevices(_ completionHandler: @escaping @Sendable (Result<[Device], Error>) -> Void) -> Cancellable {
        devicesProxy.getDevices(
            accountNumber: accountNumber,
            retryStrategy: .default,
            completion: completionHandler
        )
    }

    @discardableResult
    func deleteDevice(
        _ identifier: String,
        completionHandler: @escaping @Sendable (Result<Bool, Error>) -> Void
    ) -> Cancellable {
        devicesProxy.deleteDevice(
            accountNumber: accountNumber,
            identifier: identifier,
            retryStrategy: .default,
            completion: completionHandler
        )
    }
}

class MockDeviceManaging: DeviceManaging {
    let currentDeviceId: String? = "123"
    let getDevicesCompletionHandler: (() -> Result<[Device], Error>)?
    static private let mockDevices = [
        Device(
            id: "123",
            name: "Blind Mole",
            pubkey: PrivateKey().publicKey,
            hijackDNS: false,
            created: Date(),
            ipv4Address: IPAddressRange(from: "127.0.0.1/32")!,
            ipv6Address: IPAddressRange(from: "::ff/64")!
        ),
        Device(
            id: "456",
            name: "Tall Mole",
            pubkey: PrivateKey().publicKey,
            hijackDNS: false,
            created: Date(),
            ipv4Address: IPAddressRange(from: "127.0.0.1/32")!,
            ipv6Address: IPAddressRange(from: "::ff/64")!
        ),
        Device(
            id: "543",
            name: "Old Mole",
            pubkey: PrivateKey().publicKey,
            hijackDNS: false,
            created: Date(),
            ipv4Address: IPAddressRange(from: "127.0.0.1/32")!,
            ipv6Address: IPAddressRange(from: "::ff/64")!
        ),
        Device(
            id: "867",
            name: "Young Mole",
            pubkey: PrivateKey().publicKey,
            hijackDNS: false,
            created: Date(),
            ipv4Address: IPAddressRange(from: "127.0.0.1/32")!,
            ipv6Address: IPAddressRange(from: "::ff/64")!
        ),
        Device(
            id: "234",
            name: "Rich Mole",
            pubkey: PrivateKey().publicKey,
            hijackDNS: false,
            created: Date(),
            ipv4Address: IPAddressRange(from: "127.0.0.1/32")!,
            ipv6Address: IPAddressRange(from: "::ff/64")!
        ),
    ]
    let devicesToReturn: Int
    init(
        devicesToReturn: Int = 5,
        getDevicesCompletionHandler: (() -> Result<[Device], Error>)? = {
            .success(mockDevices)
        }
    ) {
        self.devicesToReturn = devicesToReturn
        self.getDevicesCompletionHandler = getDevicesCompletionHandler
    }

    func deleteDevice(
        _ identifier: String,
        completionHandler: @escaping @Sendable (Result<Bool, any Error>) -> Void
    ) -> any MullvadTypes.Cancellable {
        DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
            completionHandler(.success(true))
        }
        return AnyCancellable()
    }

    func getDevices(_ completionHandler: @escaping @Sendable (Result<[Device], Error>) -> Void) -> Cancellable {
        if let getDevicesCompletionHandler {
            completionHandler(getDevicesCompletionHandler().map { Array($0.prefix(devicesToReturn)) })
        }
        return AnyCancellable()
    }
}
