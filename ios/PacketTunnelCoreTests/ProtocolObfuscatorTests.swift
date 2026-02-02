//
//  ProtocolObfuscatorTests.swift
//  PacketTunnelCoreTests
//
//  Created by Marco Nikic on 2023-11-21.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Network
import XCTest

@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes
@testable import PacketTunnelCore

final class ProtocolObfuscatorTests: XCTestCase {
    var obfuscator: ProtocolObfuscator<TunnelObfuscationStub>!

    override func setUpWithError() throws {
        obfuscator = ProtocolObfuscator<TunnelObfuscationStub>()
    }

    private func makeEndpoint(obfuscation: ObfuscationMethod) throws -> SelectedEndpoint {
        let address = try XCTUnwrap(IPv4Address("1.2.3.4"))
        let gateway = try XCTUnwrap(IPv4Address("5.6.7.8"))
        let v4Endpoint = IPv4Endpoint(ip: address, port: 56)

        return SelectedEndpoint(
            socketAddress: .ipv4(v4Endpoint),
            ipv4Gateway: gateway,
            ipv6Gateway: .any,
            publicKey: Data(),
            obfuscation: obfuscation
        )
    }

    func testObfuscateOffDoesNotChangeEndpoint() throws {
        let endpoint = try makeEndpoint(obfuscation: .off)
        let nonObfuscated = obfuscator.obfuscate(endpoint)

        XCTAssertEqual(endpoint, nonObfuscated.endpoint)
    }

    func testObfuscateUdpOverTcp() throws {
        let endpoint = try makeEndpoint(obfuscation: .udpOverTcp)
        let obfuscated = obfuscator.obfuscate(endpoint)
        let obfuscationProtocol = try XCTUnwrap(obfuscator.tunnelObfuscator as? TunnelObfuscationStub)

        validate(obfuscated.endpoint, against: obfuscationProtocol)
    }

    func testObfuscateShadowsocks() throws {
        let endpoint = try makeEndpoint(obfuscation: .shadowsocks)
        let obfuscated = obfuscator.obfuscate(endpoint)
        let obfuscationProtocol = try XCTUnwrap(obfuscator.tunnelObfuscator as? TunnelObfuscationStub)

        validate(obfuscated.endpoint, against: obfuscationProtocol)
    }

    func testObfuscateQuic() throws {
        let endpoint = try makeEndpoint(obfuscation: .quic(hostname: "test.mullvad.net", token: "token"))
        let obfuscated = obfuscator.obfuscate(endpoint)
        let obfuscationProtocol = try XCTUnwrap(obfuscator.tunnelObfuscator as? TunnelObfuscationStub)

        validate(obfuscated.endpoint, against: obfuscationProtocol)
    }

    func testObfuscateLwo() throws {
        let endpoint = try makeEndpoint(obfuscation: .lwo)
        let obfuscated = obfuscator.obfuscate(endpoint)
        let obfuscationProtocol = try XCTUnwrap(obfuscator.tunnelObfuscator as? TunnelObfuscationStub)

        validate(obfuscated.endpoint, against: obfuscationProtocol)
    }
}

extension ProtocolObfuscatorTests {
    private func validate(
        _ obfuscatedEndpoint: SelectedEndpoint,
        against obfuscationProtocol: TunnelObfuscationStub
    ) {

        guard case let .ipv4(ipv4Endpoint) = obfuscatedEndpoint.socketAddress else {
            XCTFail("Expected IPv4 endpoint after obfuscation")
            return
        }
        XCTAssertEqual(ipv4Endpoint.ip, .loopback)
        XCTAssertEqual(ipv4Endpoint.port, obfuscationProtocol.localUdpPort)
    }
}
