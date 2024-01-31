//
//  MullvadApi.swift
//  MullvadVPNUITests
//
//  Created by Emils on 31/01/2024.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation

class ApiError: Error {
    let description: String
    let kind: MullvadApiErrorKind
    init(_ result: MullvadApiError) {
        kind = result.kind
        description = String(cString: result.description)
        mullvad_api_error_drop(result)
    }

    func throwIfErr() throws {
        if self.kind.rawValue != 0 {
            throw self
        }
    }
}

class InitMutableBufferError: Error {
    let description = "Failed to allocate memory for mutable buffer"
}

class MullvadApi {
    private var clientContext: IosMullvadApiClient
    init() throws {
        clientContext = IosMullvadApiClient(ptr: nil)
        // TODO: read the API address and hostname from a config file
        let apiAddress = "45.83.223.193:443"
        let hostname = "api.mullvad.net"
        let result = mullvad_api_initialize_api_runtime(
            &clientContext,
            apiAddress,
            apiAddress.lengthOfBytes(),
            hostname,
            hostname.lengthOfBytes()
        )
        try ApiError(result).throwIfErr()
    }

    /// Removes all devices assigned to the specified account
    func removeAllDevices(forAccount: String) throws {
        let result = mullvad_api_remove_all_devices(
            clientContext,
            forAccount,
            forAccount.lengthOfBytes()
        )

        try ApiError(result).throwIfErr()
    }

    /// Public key must be at least 32 bytes long - only 32 bytes of it will be read
    func addDevice(forAccount: String, publicKey: Data) throws {
        let result = mullvad_api_add_device(
            clientContext,
            forAccount,
            forAccount.lengthOfBytes(),
            (publicKey as NSData).bytes
        )

        try ApiError(result).throwIfErr()
    }

    /// Returns a unix timestamp of the expiry date for the specified account.
    func getExpiry(forAccount: String) throws -> UInt64 {
        var expiry = UInt64(0)
        let result = mullvad_api_get_expiry(clientContext, forAccount, forAccount.lengthOfBytes(), &expiry)

        try ApiError(result).throwIfErr()

        return expiry
    }

    func createAccount() throws -> String {
        guard let data = NSMutableData(length: 128) else {
            throw InitMutableBufferError()
        }

        var dataLen = data.count
        let result = mullvad_api_create_account(
            clientContext,
            data.mutableBytes.assumingMemoryBound(to: UInt8.self),
            &dataLen
        )
        try ApiError(result).throwIfErr()

        let newAccount = String(cString: data.mutableBytes.assumingMemoryBound(to: Int8.self))

        return newAccount
    }

    func listDevices(forAccount: String) throws -> [Device] {
        var iterator = MullvadApiDeviceIterator()
        let result = mullvad_api_list_devices(clientContext, forAccount, forAccount.lengthOfBytes(), &iterator)
        try ApiError(result).throwIfErr()

        return DeviceIterator(iter: iterator).collect()
    }

    func delete(account: String) throws {
        let result = mullvad_api_delete_account(clientContext, account, account.lengthOfBytes())
        try ApiError(result).throwIfErr()
    }

    deinit {
        mullvad_api_runtime_drop(clientContext)
    }

    class Device {
        let name: String
        let id: UUID

        init(device_struct: MullvadApiDevice) {
            let deviceNameData = NSData(bytes: device_struct.name_ptr, length: Int(device_struct.name_len))
            name = String(bytes: deviceNameData, encoding: .utf8)!
            id = UUID(uuid: device_struct.id)
        }
    }

    private class DeviceIterator {
        private let backingIter: MullvadApiDeviceIterator

        init(iter: MullvadApiDeviceIterator) {
            backingIter = iter
        }

        func collect() -> [Device] {
            var nextDevice = MullvadApiDevice()
            var devices: [Device] = []
            while mullvad_api_device_iter_next(backingIter, &nextDevice) {
                devices.append(Device(device_struct: nextDevice))
                mullvad_api_device_drop(nextDevice)
            }
            return devices
        }

        deinit {
            mullvad_api_device_iter_drop(backingIter)
        }
    }
}

private extension String {
    func lengthOfBytes() -> UInt {
        return UInt(self.lengthOfBytes(using: String.Encoding.utf8))
    }
}
