//
//  MullvadApi.swift
//  MullvadVPNUITests
//
//  Created by Emils on 31/01/2024.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation

struct ApiError: Error {
    let description: String
    let kind: MullvadApiErrorKind
    init(_ result: MullvadApiError) {
        kind = result.kind
        if result.description != nil {
            description = String(cString: result.description)
        } else {
            description = "No error"
        }
        mullvad_api_error_drop(result)
    }

    func throwIfErr() throws {
        if self.kind.rawValue != 0 {
            throw self
        }
    }
}

struct InitMutableBufferError: Error {
    let description = "Failed to allocate memory for mutable buffer"
}

/// - Warning: Do not change the `apiAddress` or the `hostname` after the time `MullvadApi.init` has been invoked
/// The Mullvad API crate is using a global static variable to store those. They will be initialized only once.
///
class MullvadApi {
    private var clientContext = MullvadApiClient()

    init(apiAddress: String, hostname: String) throws {
        let result = mullvad_api_client_initialize(
            &clientContext,
            apiAddress,
            hostname
        )
        try ApiError(result).throwIfErr()
    }

    /// Removes all devices assigned to the specified account
    func removeAllDevices(forAccount: String) throws {
        let result = mullvad_api_remove_all_devices(
            clientContext,
            forAccount
        )

        try ApiError(result).throwIfErr()
    }

    /// Public key must be at least 32 bytes long - only 32 bytes of it will be read
    func addDevice(forAccount: String, publicKey: Data) throws {
        var device = MullvadApiDevice()
        let result = mullvad_api_add_device(
            clientContext,
            forAccount,
            (publicKey as NSData).bytes,
            &device
        )

        try ApiError(result).throwIfErr()
    }

    /// Returns a unix timestamp of the expiry date for the specified account.
    func getExpiry(forAccount: String) throws -> UInt64 {
        var expiry = UInt64(0)
        let result = mullvad_api_get_expiry(clientContext, forAccount, &expiry)

        try ApiError(result).throwIfErr()

        return expiry
    }

    func createAccount() throws -> String {
        var newAccountPtr: UnsafePointer<CChar>?
        let result = mullvad_api_create_account(
            clientContext,
            &newAccountPtr
        )
        try ApiError(result).throwIfErr()

        let newAccount = String(cString: newAccountPtr!)
        return newAccount
    }

    func listDevices(forAccount: String) throws -> [Device] {
        var iterator = MullvadApiDeviceIterator()
        let result = mullvad_api_list_devices(clientContext, forAccount, &iterator)
        try ApiError(result).throwIfErr()

        return DeviceIterator(iter: iterator).collect()
    }

    func delete(account: String) throws {
        let result = mullvad_api_delete_account(clientContext, account)
        try ApiError(result).throwIfErr()
    }

    deinit {
        mullvad_api_client_drop(clientContext)
    }

    struct Device {
        let name: String
        let id: UUID

        init(device_struct: MullvadApiDevice) {
            name = String(cString: device_struct.name_ptr)
            id = UUID(uuid: device_struct.id)
        }
    }

    class DeviceIterator {
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
