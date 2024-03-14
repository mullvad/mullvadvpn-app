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
@testable import WireGuardKitTypes
import XCTest

final class ProtocolObfuscatorTests: XCTestCase {
    var obfuscator: ProtocolObfuscator<TunnelObfuscationStub>!
    var address: IPv4Address!
    var gateway: IPv4Address!
    var v4Endpoint: IPv4Endpoint!
    var endpoint: MullvadEndpoint!

    override func setUpWithError() throws {
        obfuscator = ProtocolObfuscator<TunnelObfuscationStub>()
        address = try XCTUnwrap(IPv4Address("1.2.3.4"))
        gateway = try XCTUnwrap(IPv4Address("5.6.7.8"))
        v4Endpoint = IPv4Endpoint(ip: address, port: 56)
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

    func testObfuscateOnPort80() throws {
        let settings = settings(.on, obfuscationPort: .port80)
        let obfuscatedEndpoint = obfuscator.obfuscate(endpoint, settings: settings)
        let obfuscationProtocol = try XCTUnwrap(obfuscator.tunnelObfuscator as? TunnelObfuscationStub)

        validate(obfuscatedEndpoint, against: obfuscationProtocol, expect: .port80)
    }

    func testObfuscateOnPort5001() throws {
        let settings = settings(.on, obfuscationPort: .port5001)
        let obfuscatedEndpoint = obfuscator.obfuscate(endpoint, settings: settings)
        let obfuscationProtocol = try XCTUnwrap(obfuscator.tunnelObfuscator as? TunnelObfuscationStub)

        validate(obfuscatedEndpoint, against: obfuscationProtocol, expect: .port5001)
    }

    func testObfuscateOnPortAutomaticIsPort80OnEvenRetryAttempts() throws {
        let settings = settings(.on, obfuscationPort: .automatic)
        let obfuscatedEndpoint = obfuscator.obfuscate(endpoint, settings: settings, retryAttempts: 2)
        let obfuscationProtocol = try XCTUnwrap(obfuscator.tunnelObfuscator as? TunnelObfuscationStub)

        validate(obfuscatedEndpoint, against: obfuscationProtocol, expect: .port80)
    }

    func testObfuscateOnPortAutomaticIsPort5001OnOddRetryAttempts() throws {
        let settings = settings(.on, obfuscationPort: .automatic)
        let obfuscatedEndpoint = obfuscator.obfuscate(endpoint, settings: settings, retryAttempts: 3)
        let obfuscationProtocol = try XCTUnwrap(obfuscator.tunnelObfuscator as? TunnelObfuscationStub)

        validate(obfuscatedEndpoint, against: obfuscationProtocol, expect: .port5001)
    }

    func testObfuscateAutomaticIsPort80EveryThirdAttempts() throws {
        let settings = settings(.automatic, obfuscationPort: .automatic)
        let obfuscatedEndpoint = obfuscator.obfuscate(endpoint, settings: settings, retryAttempts: 6)
        let obfuscationProtocol = try XCTUnwrap(obfuscator.tunnelObfuscator as? TunnelObfuscationStub)

        validate(obfuscatedEndpoint, against: obfuscationProtocol, expect: .port80)
    }

    func testObfuscateAutomaticIsPort5001EveryFourthAttempts() throws {
        let settings = settings(.automatic, obfuscationPort: .automatic)
        let obfuscatedEndpoint = obfuscator.obfuscate(endpoint, settings: settings, retryAttempts: 7)
        let obfuscationProtocol = try XCTUnwrap(obfuscator.tunnelObfuscator as? TunnelObfuscationStub)

        validate(obfuscatedEndpoint, against: obfuscationProtocol, expect: .port5001)
    }

    private func validate(
        _ obfuscatedEndpoint: MullvadEndpoint,
        against obfuscationProtocol: TunnelObfuscationStub,
        expect port: WireGuardObfuscationPort
    ) {
        XCTAssertEqual(obfuscatedEndpoint.ipv4Relay.ip, .loopback)
        XCTAssertEqual(obfuscatedEndpoint.ipv4Relay.port, obfuscationProtocol.localUdpPort)
        XCTAssertEqual(obfuscationProtocol.remotePort, port.portValue)
    }

    private func settings(
        _ obfuscationState: WireGuardObfuscationState,
        obfuscationPort: WireGuardObfuscationPort
    ) -> Settings {
        Settings(
            privateKey: PrivateKey(),
            interfaceAddresses: [IPAddressRange(from: "127.0.0.1/32")!],
            relayConstraints: RelayConstraints(),
            dnsServers: .gateway,
            obfuscation: WireGuardObfuscationSettings(
                state: obfuscationState,
                port: obfuscationPort
            ), tunnelQuantumResistance: .automatic
        )
    }
}
