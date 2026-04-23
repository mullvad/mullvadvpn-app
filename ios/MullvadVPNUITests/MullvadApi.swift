//
//  MullvadApi.swift
//  MullvadVPNUITests
//
//  Created by Emils on 31/01/2024.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadRustRuntime

struct MullvadApiError: Error {
    let description: String
}

struct Device {
    let name: String
    let id: UUID
}

private struct NewAccountResponse: Decodable { let number: String }
private struct AccountResponse: Decodable { let expiry: Date }
private struct DeviceResponse: Decodable {
    let id: String
    let name: String
}

/// - Warning: Do not change the `apiAddress` or the `hostname` after the time `MullvadApi.init` has been invoked.
class MullvadApi {
    private let context: SwiftApiContext

    private static let logger = Logger(label: "MullvadApi")

    init(apiAddress: String, hostname: String) throws {
        Self.logger.debug("Initializing MullvadApi with address: \(apiAddress), hostname: \(hostname)")
        let directRaw = convert_builtin_access_method_setting(
            UUID().uuidString, "Direct", true, UInt8(KindDirect.rawValue), nil
        )
        // Bridges and EncryptedDNS must be disabled because the shadowsocks bridge provider
        // is initialized with a nil loader. If Direct fails and the access method selector
        // falls back to Bridges, it will dereference the nil pointer and SIGABRT.
        let bridgesRaw = convert_builtin_access_method_setting(
            UUID().uuidString, "Bridges", false, UInt8(KindBridge.rawValue), nil
        )
        let encryptedDNSRaw = convert_builtin_access_method_setting(
            UUID().uuidString,
            "EncryptedDNS",
            false,
            UInt8(
                KindEncryptedDnsProxy.rawValue
            ),
            nil
        )
        let domainFrontingRaw = convert_builtin_access_method_setting(
            UUID().uuidString,
            "Domain fronting",
            false,
            UInt8(KindDomainFronting.rawValue),
            nil
        )
        let settingsWrapper = init_access_method_settings_wrapper(
            directRaw, bridgesRaw, encryptedDNSRaw, domainFrontingRaw, nil, 0
        )
        let bridgeProvider = SwiftShadowsocksLoaderWrapper(
            _0: SwiftShadowsocksLoaderWrapperContext(shadowsocks_loader: nil)
        )
        let domainFrontingConfig = SwiftDomainFrontingConfig(front: "", proxy_host: "")
        context = mullvad_api_init_inner(
            hostname,
            apiAddress,
            hostname,
            domainFrontingConfig,
            false,
            bridgeProvider,
            settingsWrapper,
            nil,
            nil
        )
    }

    func createAccount() throws -> String {
        let response = try makeRequest { cookie, strategy in
            mullvad_ios_create_account(context, cookie, strategy)
        }
        let data = try requireBody(response)
        return try JSONDecoder().decode(NewAccountResponse.self, from: data).number
    }

    func delete(account: String) throws {
        _ = try makeRequest { cookie, strategy in
            mullvad_ios_delete_account(context, cookie, strategy, account)
        }
    }

    func addDevice(forAccount: String, publicKey: Data) throws {
        _ = try publicKey.withUnsafeBytes { ptr -> MullvadApiResponse in
            try makeRequest { cookie, strategy in
                mullvad_ios_create_device(
                    context,
                    cookie,
                    strategy,
                    forAccount,
                    ptr.baseAddress!.assumingMemoryBound(to: UInt8.self)
                )
            }
        }
    }

    func getExpiry(forAccount: String) throws -> UInt64 {
        let response = try makeRequest { cookie, strategy in
            mullvad_ios_get_account(context, cookie, strategy, forAccount)
        }
        let data = try requireBody(response)
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        let decoded = try decoder.decode(AccountResponse.self, from: data)
        return UInt64(decoded.expiry.timeIntervalSince1970)
    }

    func listDevices(forAccount: String) throws -> [Device] {
        let response = try makeRequest { cookie, strategy in
            mullvad_ios_get_devices(context, cookie, strategy, forAccount)
        }
        let data = try requireBody(response)
        let deviceResponses = try JSONDecoder().decode([DeviceResponse].self, from: data)
        return deviceResponses.compactMap { d in
            guard let uuid = UUID(uuidString: d.id) else { return nil }
            return Device(name: d.name, id: uuid)
        }
    }

    private func requireBody(_ response: MullvadApiResponse) throws -> Data {
        guard response.success, let data = response.body else {
            throw MullvadApiError(description: response.errorDescription ?? "Request failed")
        }
        return data
    }

    @discardableResult
    private func makeRequest(
        _ call: (UnsafeMutableRawPointer, SwiftRetryStrategy) -> SwiftCancelHandle
    ) throws -> MullvadApiResponse {
        let semaphore = DispatchSemaphore(value: 0)
        nonisolated(unsafe) var apiResponse: MullvadApiResponse?

        let completion = MullvadApiCompletion { response in
            apiResponse = response
            semaphore.signal()
        }
        let cookie = Unmanaged.passRetained(completion).toOpaque()
        let strategy = mullvad_api_retry_strategy_constant(3, 1)
        var handle = call(cookie, strategy)
        semaphore.wait()
        mullvad_api_cancel_task_drop(&handle)

        guard let response = apiResponse else {
            throw MullvadApiError(description: "No response received")
        }
        return response
    }
}
