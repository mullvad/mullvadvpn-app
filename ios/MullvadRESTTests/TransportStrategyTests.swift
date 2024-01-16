//
//  TransportStrategyTests.swift
//  MullvadRESTTests
//
//  Created by Marco Nikic on 2023-04-27.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes
import XCTest

class TransportStrategyTests: XCTestCase {
    var userDefaults: UserDefaults!
    static var suiteName: String!

    private var directAccess: PersistentAccessMethod!
    private var bridgeAccess: PersistentAccessMethod!

    private var shadowsocksLoader: ShadowsocksLoaderStub!

    override class func setUp() {
        super.setUp()
        suiteName = UUID().uuidString
    }

    override func setUpWithError() throws {
        try super.setUpWithError()
        userDefaults = UserDefaults(suiteName: Self.suiteName)

        shadowsocksLoader = ShadowsocksLoaderStub(configuration: ShadowsocksConfiguration(
            address: .ipv4(.loopback),
            port: 1080,
            password: "123",
            cipher: CipherIdentifiers.CHACHA20.description
        ))

        directAccess = PersistentAccessMethod(
            id: UUID(uuidString: "C9DB7457-2A55-42C3-A926-C07F82131994")!,
            name: "",
            isEnabled: true,
            proxyConfiguration: .direct
        )

        bridgeAccess = PersistentAccessMethod(
            id: UUID(uuidString: "8586E75A-CA7B-4432-B70D-EE65F3F95084")!,
            name: "",
            isEnabled: true,
            proxyConfiguration: .bridges
        )
    }

    override func tearDownWithError() throws {
        userDefaults.removePersistentDomain(forName: Self.suiteName)
        try super.tearDownWithError()
    }

    func testDefaultStrategyIsDirectWhenAllMethodsAreDisabled() throws {
        directAccess.isEnabled = false
        bridgeAccess.isEnabled = false
        let transportStrategy = TransportStrategy(
            userDefaults,
            datasource: AccessMethodRepositoryStub(accessMethods: [
                directAccess,
                bridgeAccess,
            ]),
            shadowsocksLoader: shadowsocksLoader
        )
        for _ in 0 ... 4 {
            transportStrategy.didFail()
            XCTAssertEqual(transportStrategy.connectionTransport(), .direct)
        }
    }

    func testReuseSameStrategyWhenEverythingElseIsDisabled() throws {
        directAccess.isEnabled = false
        let transportStrategy = TransportStrategy(
            userDefaults,
            datasource: AccessMethodRepositoryStub(accessMethods: [
                directAccess,
                bridgeAccess,
            ]),
            shadowsocksLoader: shadowsocksLoader
        )

        for _ in 0 ... 10 {
            transportStrategy.didFail()

            XCTAssertEqual(
                transportStrategy.connectionTransport(),
                .shadowsocks(configuration: try XCTUnwrap(shadowsocksLoader.load()))
            )
        }
    }

    func testLoopsFromTheStartAfterTryingAllEnabledStrategies() {
        let transportStrategy = TransportStrategy(
            userDefaults,
            datasource: AccessMethodRepositoryStub(accessMethods: [
                directAccess,
                bridgeAccess,
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
            ]),
            shadowsocksLoader: shadowsocksLoader
        )
        let accessMethodsCount = 3
        for i in 0 ..< (accessMethodsCount * 2) {
            let previousOne = transportStrategy.connectionTransport()
            transportStrategy.didFail()
            let currentOne = transportStrategy.connectionTransport()
            if i % accessMethodsCount == 0 {
                XCTAssertEqual(previousOne, .direct)
            } else {
                XCTAssertNotEqual(previousOne, currentOne)
            }
        }
    }

    func testUsesNextWhenItIsNotReachable() {
        bridgeAccess.isEnabled = false
        let transportStrategy = TransportStrategy(
            userDefaults,
            datasource: AccessMethodRepositoryStub(accessMethods: [
                directAccess,
                bridgeAccess,
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
            ]),
            shadowsocksLoader: shadowsocksLoader
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

    func testGoToNextStrategyWhenItFailsToLoadBridgeConfiguration() {
        shadowsocksLoader.error = IOError.fileNotFound
        let transportStrategy = TransportStrategy(
            userDefaults,
            datasource: AccessMethodRepositoryStub(accessMethods: [
                directAccess,
                bridgeAccess,
            ]),
            shadowsocksLoader: shadowsocksLoader
        )

        transportStrategy.didFail()
        XCTAssertEqual(transportStrategy.connectionTransport(), .direct)
    }

    func testNoLoopOnFailureAtLoadingConfigurationWhenBridgeIsOnlyEnabled() {
        shadowsocksLoader.error = IOError.fileNotFound
        directAccess.isEnabled = false
        let transportStrategy = TransportStrategy(
            userDefaults,
            datasource: AccessMethodRepositoryStub(accessMethods: [
                directAccess,
                bridgeAccess,
            ]),
            shadowsocksLoader: shadowsocksLoader
        )
        for _ in 0 ... 10 {
            transportStrategy.didFail()
            XCTAssertEqual(transportStrategy.connectionTransport(), .none)
        }
    }

    func testUsesSocks5WithAuthenticationWhenItReaches() throws {
        let username = "user"
        let password = "pass"
        let authentication = PersistentProxyConfiguration.SocksAuthentication.usernamePassword(
            username: username,
            password: password
        )
        let socks5Configuration = PersistentProxyConfiguration.SocksConfiguration(
            server: .ipv4(.loopback),
            port: 1080,
            authentication: authentication
        )
        let transportStrategy = TransportStrategy(
            userDefaults,
            datasource: AccessMethodRepositoryStub(accessMethods: [
                directAccess,
                bridgeAccess,
                PersistentAccessMethod(
                    id: UUID(),
                    name: "",
                    isEnabled: true,
                    proxyConfiguration: .socks5(socks5Configuration)
                ),
            ]),
            shadowsocksLoader: shadowsocksLoader
        )

        XCTAssertEqual(transportStrategy.connectionTransport(), .direct)
        transportStrategy.didFail()

        XCTAssertEqual(
            transportStrategy.connectionTransport(),
            .shadowsocks(configuration: try XCTUnwrap(shadowsocksLoader.load()))
        )
        transportStrategy.didFail()

        guard case let .socks5(configuration) = transportStrategy.connectionTransport(),
              username == configuration.username,
              password == configuration.password else {
            XCTAssertThrowsError("Failed to load Socks5 with authentication")
            return
        }
    }
}

extension TransportStrategy.Transport: Equatable {
    public static func == (lhs: Self, rhs: Self) -> Bool {
        switch (lhs, rhs) {
        case(.direct, .direct), (.none, .none):
            return true
        case let (.shadowsocks(config1), .shadowsocks(config2)):
            return config1.port == config2.port && config1.cipher == config2.cipher && config1.password == config2
                .password
        case let (.socks5(config1), .socks5(config2)):
            return config1.proxyEndpoint == config2.proxyEndpoint && config1.username == config2.username && config1
                .password == config2.password
        default:
            return false
        }
    }
}

private enum IOError: Error {
    case fileNotFound
}
