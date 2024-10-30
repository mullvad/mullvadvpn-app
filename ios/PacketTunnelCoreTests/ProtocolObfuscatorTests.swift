//
//  ProtocolObfuscatorTests.swift
//  PacketTunnelCoreTests
//
//  Created by Marco Nikic on 2023-11-21.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadSettings
@testable import MullvadTypes
import Network
@testable import PacketTunnelCore
import XCTest

final class ProtocolObfuscatorTests: XCTestCase {
    var obfuscator: ProtocolObfuscator<TunnelObfuscationStub>!
    var endpoint: MullvadEndpoint!

    override func setUpWithError() throws {
        let address = try XCTUnwrap(IPv4Address("1.2.3.4"))
        let gateway = try XCTUnwrap(IPv4Address("5.6.7.8"))
        let v4Endpoint = IPv4Endpoint(ip: address, port: 56)

        obfuscator = ProtocolObfuscator<TunnelObfuscationStub>()

        endpoint = MullvadEndpoint(
            ipv4Relay: v4Endpoint,
            ipv4Gateway: gateway,
            ipv6Gateway: .any,
            publicKey: Data()
        )
    }

    func testObfuscateOffDoesNotChangeEndpoint() {
        let settings = settings(.off, obfuscationPort: .automatic)
        let nonObfuscatedEndpoint = obfuscator.obfuscate(endpoint, settings: settings)

        XCTAssertEqual(endpoint, nonObfuscatedEndpoint)
    }

    func testObfuscateUdpOverTcp() throws {
        let settings = settings(.udpOverTcp, obfuscationPort: .automatic)
        let obfuscatedEndpoint = obfuscator.obfuscate(endpoint, settings: settings)
        let obfuscationProtocol = try XCTUnwrap(obfuscator.tunnelObfuscator as? TunnelObfuscationStub)

        validate(obfuscatedEndpoint, against: obfuscationProtocol)
    }

    func testObfuscateShadowsocks() throws {
        let settings = settings(.shadowsocks, obfuscationPort: .automatic)
        let obfuscatedEndpoint = obfuscator.obfuscate(endpoint, settings: settings)
        let obfuscationProtocol = try XCTUnwrap(obfuscator.tunnelObfuscator as? TunnelObfuscationStub)

        validate(obfuscatedEndpoint, against: obfuscationProtocol)
    }
}

extension ProtocolObfuscatorTests {
    private func validate(
        _ obfuscatedEndpoint: MullvadEndpoint,
        against obfuscationProtocol: TunnelObfuscationStub
    ) {
        XCTAssertEqual(obfuscatedEndpoint.ipv4Relay.ip, .loopback)
        XCTAssertEqual(obfuscatedEndpoint.ipv4Relay.port, obfuscationProtocol.localUdpPort)
    }

    private func settings(
        _ obfuscationState: WireGuardObfuscationState,
        obfuscationPort: WireGuardObfuscationUdpOverTcpPort
    ) -> LatestTunnelSettings {
        LatestTunnelSettings(wireGuardObfuscation: WireGuardObfuscationSettings(
            state: obfuscationState,
            udpOverTcpPort: obfuscationPort
        ))
    }
}
