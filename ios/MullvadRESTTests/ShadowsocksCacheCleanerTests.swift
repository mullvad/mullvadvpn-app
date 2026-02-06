//
//  ShadowsocksCacheCleanerTests.swift
//  MullvadRESTTests
//
//  Created by Marco Nikic on 2025-09-18.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Network
import Testing

@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes

actor ShadowsocksCacheCleanerTests {
    var cache = ShadowsocksCacheStub(
        configuration:
            ShadowsocksConfiguration(
                address: .ipv4(IPv4Address.loopback),
                port: 1234,
                password: "password",
                cipher: "chacha20"
            )
    )

    deinit {
        cache.onRead = nil
        cache.onWrite = nil
        cache.onClear = nil
    }

    @Test func storesLastAccessMethodUUID() async throws {
        let cacheCleaner = ShadowsocksCacheCleaner(cache: cache)
        let newMethodUUID = UUID()

        cacheCleaner.accessMethodChangedTo(newMethodUUID)
        #expect(newMethodUUID == cacheCleaner.lastChangedUUID)
    }

    @Test func clearsCacheWhenPreviousChangeWasShadowsocksUUID() async throws {
        let bridges = AccessMethodRepository.bridgeId
        let direct = AccessMethodRepository.directId

        await confirmation("Did clear cache") { didClearCache in
            cache.onClear = {
                didClearCache()
            }
            let cacheCleaner = ShadowsocksCacheCleaner(cache: cache)
            cacheCleaner.accessMethodChangedTo(bridges)
            cacheCleaner.accessMethodChangedTo(direct)
        }
    }

    @Test func doesNotClearCacheWhenOtherMethodsChange() async throws {
        let encryptedDNS = AccessMethodRepository.encryptedDNSId
        let direct = AccessMethodRepository.directId

        await confirmation("Did clear cache", expectedCount: 0) { didClearCache in
            let cacheCleaner = ShadowsocksCacheCleaner(cache: cache)
            cache.onClear = {
                didClearCache()
            }
            cacheCleaner.accessMethodChangedTo(encryptedDNS)
            cacheCleaner.accessMethodChangedTo(direct)
        }
    }
}

struct ShadowsocksCacheStub: ShadowsocksConfigurationCacheProtocol {
    let configuration: ShadowsocksConfiguration

    var onRead: (@Sendable () -> Void)?
    var onWrite: (@Sendable () -> Void)?
    var onClear: (@Sendable () -> Void)?

    func read() throws -> ShadowsocksConfiguration {
        onRead?()
        return configuration
    }

    func write(_ configuration: ShadowsocksConfiguration) throws {
        onWrite?()
    }

    func clear() throws {
        onClear?()
    }
}
