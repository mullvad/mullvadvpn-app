//
//  ProtocolObfuscatorTests.swift
//  PacketTunnelCoreTests
//
//  Created by Marco Nikic on 2023-11-21.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Network
import WireGuardKitTypes
import XCTest

@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes
@testable import PacketTunnelCore

final class ProtocolObfuscatorTests: XCTestCase {
    var obfuscator: ProtocolObfuscator<TunnelObfuscationStub>!
    var endpoint: MullvadEndpoint!
    let testPublicKey = PrivateKey().publicKey

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
        let nonObfuscated = obfuscator.obfuscate(
            endpoint,
            relayFeatures: nil,
            obfuscationMethod: .off,
            clientPublicKey: testPublicKey
        )

        XCTAssertEqual(endpoint, nonObfuscated.endpoint)
    }

    func testObfuscateUdpOverTcp() throws {
        let obfuscated = obfuscator.obfuscate(
            endpoint,
            relayFeatures: nil,
            obfuscationMethod: .udpOverTcp,
            clientPublicKey: testPublicKey
        )
        let obfuscationProtocol = try XCTUnwrap(obfuscator.tunnelObfuscator as? TunnelObfuscationStub)

        validate(obfuscated.endpoint, against: obfuscationProtocol)
    }

    func testObfuscateShadowsocks() throws {
        let obfuscated = obfuscator.obfuscate(
            endpoint,
            relayFeatures: nil,
            obfuscationMethod: .shadowsocks,
            clientPublicKey: testPublicKey
        )
        let obfuscationProtocol = try XCTUnwrap(obfuscator.tunnelObfuscator as? TunnelObfuscationStub)

        validate(obfuscated.endpoint, against: obfuscationProtocol)
    }

    func testObfuscateQuic() throws {
        let obfuscated = obfuscator.obfuscate(
            endpoint,
            relayFeatures: .init(daita: nil, quic: .init(addrIn: [], domain: "", token: "")),
            obfuscationMethod: .quic,
            clientPublicKey: testPublicKey
        )
        let obfuscationProtocol = try XCTUnwrap(obfuscator.tunnelObfuscator as? TunnelObfuscationStub)

        validate(obfuscated.endpoint, against: obfuscationProtocol)
    }

    func testObfuscateAutomaticDoesNotObfuscate() throws {
        let obfuscated = obfuscator.obfuscate(
            endpoint,
            relayFeatures: .init(daita: nil, quic: .init(addrIn: [], domain: "", token: "")),
            obfuscationMethod: .automatic,
            clientPublicKey: testPublicKey
        )

        XCTAssertEqual(endpoint, obfuscated.endpoint)
        XCTAssertEqual(.off, obfuscated.method)
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
        LatestTunnelSettings(
            wireGuardObfuscation: WireGuardObfuscationSettings(
                state: obfuscationState,
                udpOverTcpPort: obfuscationPort
            ))
    }
}
