//
//  APIAccessMethodsTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2023-12-13.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadSettings
import XCTest

final class APIAccessMethodsTests: XCTestCase {
    static let store = InMemorySettingsStore<SettingNotFound>()

    override class func setUp() {
        SettingsManager.unitTestStore = store
    }

    override class func tearDown() {
        SettingsManager.unitTestStore = nil
    }

    override func tearDownWithError() throws {
        let repository = AccessMethodRepository.shared
        repository.fetchAll().forEach {
            repository.delete(id: $0.id)
        }
    }

    func testDefaultAccessMethodsExist() throws {
        let storedMethods = AccessMethodRepository.shared.fetchAll()

        let hasDirectMethod = storedMethods.contains { method in
            method.kind == .direct
        }

        let hasBridgesMethod = storedMethods.contains { method in
            method.kind == .bridges
        }

        XCTAssertEqual(storedMethods.count, 2)
        XCTAssertTrue(hasDirectMethod && hasBridgesMethod)
    }

    func testAddingSocks5AccessMethod() throws {
        let uuid = UUID()
        let methodToStore = socks5AccessMethod(with: uuid)

        AccessMethodRepository.shared.save(methodToStore)
        let storedMethod = AccessMethodRepository.shared.fetch(by: uuid)

        XCTAssertEqual(methodToStore.id, storedMethod?.id)
    }

    func testAddingShadowSocksAccessMethod() throws {
        let uuid = UUID()
        let methodToStore = shadowsocksAccessMethod(with: uuid)

        AccessMethodRepository.shared.save(methodToStore)
        let storedMethod = AccessMethodRepository.shared.fetch(by: uuid)

        XCTAssertEqual(methodToStore.id, storedMethod?.id)
    }

    func testAddingDuplicateAccessMethodDoesNothing() throws {
        let methodToStore = socks5AccessMethod(with: UUID())

        AccessMethodRepository.shared.save(methodToStore)
        AccessMethodRepository.shared.save(methodToStore)
        let storedMethods = AccessMethodRepository.shared.fetchAll()

        // Account for .direct and .bridges that are always added by default.
        XCTAssertEqual(storedMethods.count, 3)
    }

    func testUpdatingAccessMethod() throws {
        let uuid = UUID()
        var methodToStore = socks5AccessMethod(with: uuid)

        AccessMethodRepository.shared.save(methodToStore)

        let newName = "Renamed method"
        methodToStore.name = newName

        AccessMethodRepository.shared.save(methodToStore)

        let storedMethod = AccessMethodRepository.shared.fetch(by: uuid)

        XCTAssertEqual(storedMethod?.name, newName)
    }

    func testDeletingAccessMethod() throws {
        let uuid = UUID()
        let methodToStore = socks5AccessMethod(with: uuid)

        AccessMethodRepository.shared.save(methodToStore)
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
