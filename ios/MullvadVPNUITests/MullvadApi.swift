// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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

private final class ShadowsocksProviderNil: ShadowsocksBridgeProvider {
    func bridge() -> MullvadRustRuntime.ShadowsocksWrapper? {
        nil
    }
}

/// - Warning: Do not change the `apiAddress` or the `hostname` after the time `MullvadApi.init` has been invoked.
class MullvadApi {
    private let context: ApiContext

    private static let logger = Logger(label: "MullvadApi")

    init(apiAddress: String, hostname: String) throws {
        Self.logger.debug("Initializing MullvadApi with address: \(apiAddress), hostname: \(hostname)")
        let direct = convertBuiltinAccessMethodSetting(
            uniqueIdentifier: UUID().uuidString,
            name: "Direct",
            isEnabled: true,
            methodKind: .kindDirect)
        // Bridges and EncryptedDNS must be disabled because the shadowsocks bridge provider
        // is initialized with a nil loader. If Direct fails and the access method selector
        // falls back to Bridges, it will dereference the nil pointer and SIGABRT.
        let bridges = convertBuiltinAccessMethodSetting(
            uniqueIdentifier: UUID().uuidString,
            name: "Bridges",
            isEnabled: false,
            methodKind: .kindBridge)
        let encryptedDNSRaw = convertBuiltinAccessMethodSetting(
            uniqueIdentifier: UUID().uuidString,
            name: "EncryptedDNS",
            isEnabled: false,
            methodKind: .kindEncryptedDnsProxy
        )
        let settingsWrapper = initAccessMethodSettingsWrapper(
            direct: direct!,
            bridges: bridges!,
            encryptedDns: encryptedDNSRaw!,
            custom: [])
        let bridgeProvider = ShadowsocksProviderNil()
        context = ApiContext(
            host: hostname,
            address: apiAddress,
            domain: hostname,
            bridgeProvider: bridgeProvider,
            settingsProvider: settingsWrapper,
            accessMethodChangeCallback: nil,
            accessMethodChangeContext: nil)
    }

    func createAccount() throws -> String {
        let response = try makeRequest { strategy in
            mullvadIosCreateAccount(apiContext: context, retryStrategy: strategy)
        }
        let data = try requireBody(response)
        return try JSONDecoder().decode(NewAccountResponse.self, from: data).number
    }

    func delete(account: String) throws {
        _ = try makeRequest { strategy in
            mullvadIosDeleteAccount(apiContext: context, retryStrategy: strategy, accountNumber: account)
        }
    }

    func addDevice(forAccount: String, publicKey: Data) throws {
        try makeRequest { strategy in
            mullvadIosCreateDevice(
                apiContext: context,
                retryStrategy: strategy,
                accountNumber: forAccount,
                publicKey: publicKey
            )
        }
    }

    func getExpiry(forAccount: String) throws -> UInt64 {
        let response = try makeRequest { strategy in
            let handle = mullvadIosGetAccount(
                apiContext: context,
                retryStrategy: strategy,
                accountNumber: forAccount)
            return handle
        }
        let data = try requireBody(response)
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        let decoded = try decoder.decode(AccountResponse.self, from: data)
        return UInt64(decoded.expiry.timeIntervalSince1970)
    }

    func listDevices(forAccount: String) throws -> [Device] {
        let response = try makeRequest { strategy in
            mullvadIosGetDevices(
                apiContext: context,
                retryStrategy: strategy,
                accountNumber: forAccount)
        }
        let data = try requireBody(response)
        let deviceResponses = try JSONDecoder().decode([DeviceResponse].self, from: data)
        return deviceResponses.compactMap { (d: DeviceResponse) -> Device? in
            guard let uuid = UUID(uuidString: d.id) else { return nil }
            return Device(name: d.name, id: uuid)
        }
    }

    private func requireBody(_ response: ApiResponse) throws -> Data {
        guard response.success(), let data = response.body() else {
            throw MullvadApiError(description: response.errorDescription() ?? "Request failed")
        }
        return data
    }

    @discardableResult
    private func makeRequest(
        _ call: (RetryStrategy) -> RequestCancelHandle
    ) throws -> ApiResponse {
        let semaphore = DispatchSemaphore(value: 0)
        nonisolated(unsafe) var apiResponse: ApiResponse?

        let completion = MullvadApiCompletion { response in
            apiResponse = response
            semaphore.signal()
        }
        let strategy = mullvadApiRetryStrategyConstant(maxRetries: 3, delaySec: 1)
        var handle = call(strategy)
        handle.startTask(completionCookie: completion)
        semaphore.wait()

        guard let response = apiResponse else {
            throw MullvadApiError(description: "No response received")
        }
        return response
    }
}
