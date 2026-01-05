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

    func testIpv6Obfuscation() throws {
        let address = try XCTUnwrap(IPv6Address("2001::1"))
        let v6Endpoint = IPv6Endpoint(ip: address, port: 56)

        let endpoint = SelectedEndpoint(
            socketAddress: .ipv6(v6Endpoint),
            ipv4Gateway: .loopback,
            ipv6Gateway: .loopback,
            publicKey: Data(),
            obfuscation: .udpOverTcp
        )

        let obfuscated = obfuscator.obfuscate(endpoint)

        // IPv6 should not skip obfuscation
        XCTAssertNotEqual(obfuscated.endpoint.socketAddress, endpoint.socketAddress)
        // IPv6 obfuscators must bind to an IPv6 loopback socket address, not IPv4
        XCTAssertEqual(
            obfuscated.endpoint.socketAddress.ip as! IPv6Address,
            IPv6Address.loopback,
        )
        XCTAssertEqual(obfuscated.endpoint.obfuscation, .udpOverTcp)
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
