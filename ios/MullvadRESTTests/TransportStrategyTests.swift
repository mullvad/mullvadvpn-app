//
//  TransportStrategyTests.swift
//  MullvadRESTTests
//
//  Created by Marco Nikic on 2023-04-27.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
import MullvadSettings
@testable import MullvadTypes
import XCTest

final class TransportStrategyTests: XCTestCase {
    var userDefaults: UserDefaults!
    static var suiteName: String!

    override class func setUp() {
        super.setUp()
        suiteName = UUID().uuidString
    }

    override func setUpWithError() throws {
        try super.setUpWithError()
        userDefaults = UserDefaults(suiteName: Self.suiteName)
    }

    override func tearDownWithError() throws {
        userDefaults.removePersistentDomain(forName: Self.suiteName)
        try super.tearDownWithError()
    }

    func testContinuesToUseDirectWhenNoOneIsEnabled() {
        let transportStrategy = TransportStrategy(
            userDefaults,
            datasource: AccessMethodRepositoryStub(accessMethods: [
                PersistentAccessMethod(
                    id: UUID(uuidString: "C9DB7457-2A55-42C3-A926-C07F82131994")!,
                    name: "",
                    isEnabled: false,
                    proxyConfiguration: .direct
                ),
                PersistentAccessMethod(
                    id: UUID(uuidString: "8586E75A-CA7B-4432-B70D-EE65F3F95084")!,
                    name: "",
                    isEnabled: false,
                    proxyConfiguration: .bridges
                ),
            ])
        )

        for _ in 0 ... 4 {
            transportStrategy.didFail()
            XCTAssertEqual(transportStrategy.connectionTransport(), .direct)
        }
    }

    func testContinuesToUseBridgeWhenJustOneIsEnabled() {
        let transportStrategy = TransportStrategy(
            userDefaults,
            datasource: AccessMethodRepositoryStub(accessMethods: [
                PersistentAccessMethod(
                    id: UUID(uuidString: "C9DB7457-2A55-42C3-A926-C07F82131994")!,
                    name: "",
                    isEnabled: false,
                    proxyConfiguration: .direct
                ),
                PersistentAccessMethod(
                    id: UUID(uuidString: "8586E75A-CA7B-4432-B70D-EE65F3F95084")!,
                    name: "",
                    isEnabled: true,
                    proxyConfiguration: .bridges
                ),
            ])
        )

        for _ in 0 ... 10 {
            transportStrategy.didFail()
            XCTAssertEqual(transportStrategy.connectionTransport(), .bridge)
        }
    }

    func testContinuesToUseDirectWhenItReachesEnd() {
        let transportStrategy = TransportStrategy(
            userDefaults,
            datasource: AccessMethodRepositoryStub(accessMethods: [
                PersistentAccessMethod(
                    id: UUID(uuidString: "C9DB7457-2A55-42C3-A926-C07F82131994")!,
                    name: "",
                    isEnabled: true,
                    proxyConfiguration: .direct
                ),
                PersistentAccessMethod(
                    id: UUID(uuidString: "8586E75A-CA7B-4432-B70D-EE65F3F95084")!,
                    name: "",
                    isEnabled: false,
                    proxyConfiguration: .bridges
                ),
                PersistentAccessMethod(
                    id: UUID(uuidString: "8586E75A-CA7B-4432-B70D-EE65F3F95090")!,
                    name: "",
                    isEnabled: true,
                    proxyConfiguration: .shadowsocks(PersistentProxyConfiguration.ShadowsocksConfiguration(
                        server: .ipv4(.loopback),
                        port: 8083,
                        password: "",
                        cipher: .default
                    ))
                ),
            ])
        )

        for _ in 0 ... 3 {
            transportStrategy.didFail()
        }
        XCTAssertEqual(transportStrategy.connectionTransport(), .direct)
    }

    func testContinuesToUseNextWhenItIsNotReachable() {
        let transportStrategy = TransportStrategy(
            userDefaults,
            datasource: AccessMethodRepositoryStub(accessMethods: [
                PersistentAccessMethod(
                    id: UUID(uuidString: "C9DB7457-2A55-42C3-A926-C07F82131994")!,
                    name: "",
                    isEnabled: true,
                    proxyConfiguration: .direct
                ),
                PersistentAccessMethod(
                    id: UUID(uuidString: "8586E75A-CA7B-4432-B70D-EE65F3F95084")!,
                    name: "",
                    isEnabled: false,
                    proxyConfiguration: .bridges
                ),
                PersistentAccessMethod(
                    id: UUID(uuidString: "8586E75A-CA7B-4432-B70D-EE65F3F95090")!,
                    name: "",
                    isEnabled: true,
                    proxyConfiguration: .shadowsocks(PersistentProxyConfiguration.ShadowsocksConfiguration(
                        server: .ipv4(.loopback),
                        port: 8083,
                        password: "",
                        cipher: .default
                    ))
                ),
            ])
        )
        XCTAssertEqual(transportStrategy.connectionTransport(), .direct)
        transportStrategy.didFail()
        XCTAssertEqual(
            transportStrategy.connectionTransport(),
            .shadowsocks(configuration: ShadowsocksConfiguration(
                address: .ipv4(.loopback),
                port: 8083,
                password: "",
                cipher: ShadowsocksCipherOptions.default.rawValue.description
            ))
        )
    }
}

extension TransportStrategy.Transport: Equatable {
    public static func == (lhs: Self, rhs: Self) -> Bool {
        switch (lhs, rhs) {
        case(.direct, .direct), (.bridge, .bridge):
            return true
        case let (.shadowsocks(config1), .shadowsocks(config2)):
            return config1.address.rawValue == config2.address.rawValue && config1.port == config2.port && config1
                .cipher == config2.cipher && config1.password == config2.password
        case let (.socks5(config1), .socks5(config2)):
            return config1.address.rawValue == config2.address.rawValue && config1.port == config2.port
        default:
            return false
        }
    }
}
