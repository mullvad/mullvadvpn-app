//
//  TransportStrategyTests.swift
//  MullvadRESTTests
//
//  Created by Marco Nikic on 2023-04-27.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadMockData
@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes
import XCTest

class TransportStrategyTests: XCTestCase {
    private var directAccess: PersistentAccessMethod!
    private var bridgeAccess: PersistentAccessMethod!
    private var encryptedDNS: PersistentAccessMethod!

    private var shadowsocksLoader: ShadowsocksLoaderStub!

    override func setUpWithError() throws {
        try super.setUpWithError()

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

        encryptedDNS = PersistentAccessMethod(
            id: UUID(uuidString: "831CB1F8-1829-42DD-B9DC-82902F298EC0")!,
            name: "Encrypted DNS proxy",
            isEnabled: true,
            proxyConfiguration: .encryptedDNS
        )
    }

    func testDefaultStrategyIsDirectWhenAllMethodsAreDisabled() throws {
        directAccess.isEnabled = false
        bridgeAccess.isEnabled = false
        encryptedDNS.isEnabled = false
        let transportStrategy = TransportStrategy(
            datasource: AccessMethodRepositoryStub(accessMethods: [
                directAccess,
                bridgeAccess,
                encryptedDNS,
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
        encryptedDNS.isEnabled = false
        let transportStrategy = TransportStrategy(
            datasource: AccessMethodRepositoryStub(accessMethods: [
                directAccess,
                bridgeAccess,
                encryptedDNS,
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
            datasource: AccessMethodRepositoryStub(accessMethods: [
                directAccess,
                bridgeAccess,
                encryptedDNS,
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
        let accessMethodsCount = 4
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
        encryptedDNS.isEnabled = false
        let transportStrategy = TransportStrategy(
            datasource: AccessMethodRepositoryStub(accessMethods: [
                directAccess,
                bridgeAccess,
                encryptedDNS,
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
            datasource: AccessMethodRepositoryStub(accessMethods: [
                directAccess,
                bridgeAccess,
                encryptedDNS,
            ]),
            shadowsocksLoader: shadowsocksLoader
        )

        transportStrategy.didFail()
        XCTAssertEqual(transportStrategy.connectionTransport(), .encryptedDNS)
    }

    func testNoLoopOnFailureAtLoadingConfigurationWhenBridgeIsOnlyEnabled() {
        shadowsocksLoader.error = IOError.fileNotFound
        directAccess.isEnabled = false
        let transportStrategy = TransportStrategy(
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
        let authentication = PersistentProxyConfiguration.SocksAuthentication
            .authentication(PersistentProxyConfiguration.UserCredential(
                username: username,
                password: password
            ))
        let socks5Configuration = PersistentProxyConfiguration.SocksConfiguration(
            server: .ipv4(.loopback),
            port: 1080,
            authentication: authentication
        )
        let transportStrategy = TransportStrategy(
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

private enum IOError: Error {
    case fileNotFound
}
