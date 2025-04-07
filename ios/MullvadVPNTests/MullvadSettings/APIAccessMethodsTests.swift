//
//  APIAccessMethodsTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2023-12-13.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
@testable import MullvadSettings
@testable import MullvadTypes
import XCTest

final class APIAccessMethodsTests: XCTestCase {
    static let store = InMemorySettingsStore<SettingNotFound>()

    override static func setUp() {
        SettingsManager.unitTestStore = store
    }

    override static func tearDown() {
        SettingsManager.unitTestStore = nil
    }

    override func tearDownWithError() throws {
        let repository = AccessMethodRepository()
        repository.fetchAll().forEach {
            repository.delete(id: $0.id)
        }
    }

    func testDefaultAccessMethodsExist() throws {
        let repository = AccessMethodRepository()
        let storedMethods = repository.fetchAll()

        let hasDirectMethod = storedMethods.contains { method in
            method.kind == .direct
        }

        let hasBridgesMethod = storedMethods.contains { method in
            method.kind == .bridges
        }

        let hasEncryptedDNS = storedMethods.contains { method in
            method.kind == .encryptedDNS
        }

        XCTAssertEqual(storedMethods.count, 3)
        XCTAssertTrue(hasDirectMethod && hasBridgesMethod && hasEncryptedDNS)
    }

    func testAddingSocks5AccessMethod() throws {
        let repository = AccessMethodRepository()

        let uuid = UUID()
        let methodToStore = socks5AccessMethod(with: uuid)
        repository.save(methodToStore)

        let storedMethod = repository.fetch(by: uuid)

        XCTAssertEqual(methodToStore.id, storedMethod?.id)
    }

    func testAddingShadowSocksAccessMethod() throws {
        let repository = AccessMethodRepository()

        let uuid = UUID()
        let methodToStore = shadowsocksAccessMethod(with: uuid)
        repository.save(methodToStore)

        let storedMethod = repository.fetch(by: uuid)

        XCTAssertEqual(methodToStore.id, storedMethod?.id)
    }

    func testAddingDuplicateAccessMethodDoesNothing() throws {
        let repository = AccessMethodRepository()

        let methodToStore = socks5AccessMethod(with: UUID())

        repository.save(methodToStore)
        repository.save(methodToStore)

        let storedMethods = repository.fetchAll()

        // Account for .direct, .bridges and .encryptedDNS that are always added by default.
        XCTAssertEqual(storedMethods.count, 4)
    }

    func testUpdatingAccessMethod() throws {
        let repository = AccessMethodRepository()

        let uuid = UUID()
        var methodToStore = socks5AccessMethod(with: uuid)
        repository.save(methodToStore)

        let newName = "Renamed method"
        methodToStore.name = newName

        repository.save(methodToStore)

        let storedMethod = repository.fetch(by: uuid)

        XCTAssertEqual(storedMethod?.name, newName)
    }

    func testDeletingAccessMethod() throws {
        let repository = AccessMethodRepository()
        let uuid = UUID()
        let methodToStore = socks5AccessMethod(with: uuid)

        repository.save(methodToStore)
        repository.delete(id: uuid)

        let storedMethod = repository.fetch(by: uuid)

        XCTAssertNil(storedMethod)
    }
}

extension APIAccessMethodsTests {
    private func socks5AccessMethod(with uuid: UUID) -> PersistentAccessMethod {
        PersistentAccessMethod(
            id: uuid,
            name: "Method",
            isEnabled: true,
            proxyConfiguration: .socks5(PersistentProxyConfiguration.SocksConfiguration(
                server: .ipv4(.any),
                port: 1,
                authentication: .noAuthentication
            ))
        )
    }

    private func shadowsocksAccessMethod(with uuid: UUID) -> PersistentAccessMethod {
        PersistentAccessMethod(
            id: uuid,
            name: "Method",
            isEnabled: true,
            proxyConfiguration: .shadowsocks(PersistentProxyConfiguration.ShadowsocksConfiguration(
                server: .ipv4(.any),
                port: 1,
                password: "Password",
                cipher: .default
            ))
        )
    }
}
