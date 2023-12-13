//
//  APIAccessMethodsTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2023-12-13.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import XCTest

final class APIAccessMethodsTests: XCTestCase {
    static let store = InMemorySettingsStore<SettingNotFound>()

    override class func setUp() {
        SettingsManager.unitTestStore = store
    }

    override class func tearDown() {
        SettingsManager.unitTestStore = nil
    }

    func testDefaultAccessMethodsExist() throws {
        let storedMethods = AccessMethodRepository.shared.fetchAll()

        let hasDirectMethod = storedMethods.contains { method in
            method.kind == .direct
        }

        let hasBridgesMethod = storedMethods.contains { method in
            method.kind == .bridges
        }

        XCTAssertTrue(hasDirectMethod && hasBridgesMethod)
    }

    func testAddingSocks5AccessMethod() throws {
        let uuid = UUID()
        let methodToStore = socks5AccessMethod(with: uuid)

        AccessMethodRepository.shared.add(methodToStore)
        let storedMethod = AccessMethodRepository.shared.fetch(by: uuid)

        XCTAssertEqual(methodToStore.id, storedMethod?.id)
    }

    func testAddingShadowSocksAccessMethod() throws {
        let uuid = UUID()
        let methodToStore = shadowsocksAccessMethod(with: uuid)

        AccessMethodRepository.shared.add(methodToStore)
        let storedMethod = AccessMethodRepository.shared.fetch(by: uuid)

        XCTAssertEqual(methodToStore.id, storedMethod?.id)
    }

    func testAddingDuplicateAccessMethodDoesNothing() throws {
        let methodToStore = socks5AccessMethod(with: UUID())

        AccessMethodRepository.shared.add(methodToStore)
        AccessMethodRepository.shared.add(methodToStore)
        let storedMethods = AccessMethodRepository.shared.fetchAll()

        // Account for .direct and .bridges that are always added by default.
        XCTAssertTrue(storedMethods.count == 3)
    }

    func testUpdatingAccessMethod() throws {
        let uuid = UUID()
        var methodToStore = socks5AccessMethod(with: uuid)

        AccessMethodRepository.shared.add(methodToStore)

        let newName = "Renamed method"
        methodToStore.name = newName

        AccessMethodRepository.shared.update(methodToStore)

        let storedMethod = AccessMethodRepository.shared.fetch(by: uuid)

        XCTAssertTrue(storedMethod?.name == newName)
    }

    func testDeletingAccessMethod() throws {
        let uuid = UUID()
        let methodToStore = socks5AccessMethod(with: uuid)

        AccessMethodRepository.shared.add(methodToStore)
        AccessMethodRepository.shared.delete(id: uuid)

        let storedMethod = AccessMethodRepository.shared.fetch(by: uuid)

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
