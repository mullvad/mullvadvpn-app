//
//  RelayObfuscatorTests.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-04.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadMockData
import XCTest

@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes

final class RelayObfuscatorTests: XCTestCase {
    let defaultWireguardPort: RelayConstraint<UInt16> = .only(56)
    let defaultQuicPort: RelayConstraint<UInt16> = .only(443)

    let sampleRelays = ServerRelaysResponseStubs.sampleRelays
    var tunnelSettings = LatestTunnelSettings()

    override func setUp() {
        tunnelSettings.relayConstraints.port = defaultWireguardPort
    }

    func testObfuscateOffDoesNotChangeEndpoint() throws {
        tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(state: .off)

        let obfuscationResult = RelayObfuscator(
            relays: sampleRelays,
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: 0, obfuscationBypass: IdentityObfuscationProvider()
        ).obfuscate()

        XCTAssertEqual(obfuscationResult.port, defaultWireguardPort)
    }

    func testObfuscationForSinglehop() throws {
        let constraints = RelayConstraints(entryLocations: .any, exitLocations: .any, port: .only(5000))
        let settings = LatestTunnelSettings(
            relayConstraints: constraints,
            wireGuardObfuscation: WireGuardObfuscationSettings(
                state: .udpOverTcp,
                udpOverTcpPort: .port80
            )
        )

        let obfuscationResult = RelayObfuscator(
            relays: sampleRelays,
            tunnelSettings: settings,
            connectionAttemptCount: 0, obfuscationBypass: IdentityObfuscationProvider()
        ).obfuscate()

        let picker = SinglehopPicker(
            obfuscation: obfuscationResult,
            tunnelSettings: settings,
            connectionAttemptCount: 0
        )

        let selectedRelays = try picker.pick()

        XCTAssertNil(selectedRelays.entry)
        XCTAssertEqual(selectedRelays.exit.endpoint.ipv4Relay.port, 80)
    }

    func testObfuscationForMultihop() throws {
        let constraints = RelayConstraints(entryLocations: .any, exitLocations: .any, port: .only(5000))
        let settings = LatestTunnelSettings(
            relayConstraints: constraints,
            wireGuardObfuscation: WireGuardObfuscationSettings(
                state: .udpOverTcp,
                udpOverTcpPort: .port80
            )
        )

        let obfuscationResult = RelayObfuscator(
            relays: sampleRelays,
            tunnelSettings: settings,
            connectionAttemptCount: 0, obfuscationBypass: IdentityObfuscationProvider()
        ).obfuscate()

        let picker = MultihopPicker(
            obfuscation: obfuscationResult,
            tunnelSettings: settings,
            connectionAttemptCount: 0
        )

        let selectedRelays = try picker.pick()

        XCTAssertEqual(selectedRelays.entry?.endpoint.ipv4Relay.port, 80)
        XCTAssertEqual(selectedRelays.exit.endpoint.ipv4Relay.port, 5000)
    }

    // MARK: UdpOverTcp

    func testObfuscateUdpOverTcpPort80() throws {
        tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(
            state: .udpOverTcp,
            udpOverTcpPort: .port80
        )

        let obfuscationResult = RelayObfuscator(
            relays: sampleRelays,
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: 0, obfuscationBypass: IdentityObfuscationProvider()
        ).obfuscate()

        XCTAssertEqual(obfuscationResult.port, .only(80))
    }

    func testObfuscateUdpOverTcpPort5001() throws {
        tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(
            state: .udpOverTcp,
            udpOverTcpPort: .port5001
        )

        let obfuscationResult = RelayObfuscator(
            relays: sampleRelays,
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: 0, obfuscationBypass: IdentityObfuscationProvider()
        ).obfuscate()

        XCTAssertEqual(obfuscationResult.port, .only(5001))
    }

    func testObfuscateUpdOverTcpPortAutomaticIsRandomPort() throws {
        tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(
            state: .udpOverTcp,
            udpOverTcpPort: .automatic
        )

        (0...10).filter { $0.isMultiple(of: 2) }.forEach { attempt in
            let obfuscationResult = RelayObfuscator(
                relays: sampleRelays,
                tunnelSettings: tunnelSettings,
                connectionAttemptCount: UInt(attempt), obfuscationBypass: IdentityObfuscationProvider()
            ).obfuscate()

            let validPorts: [RelayConstraint<UInt16>] = [.only(80), .only(443), .only(5001)]
            XCTAssertTrue(validPorts.contains(obfuscationResult.port))
        }
    }

    // MARK: Shadowsocks

    func testObfuscateShadowsocksPortCustom() throws {
        tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(
            state: .shadowsocks,
            shadowsocksPort: .custom(5500)
        )

        let obfuscationResult = RelayObfuscator(
            relays: sampleRelays,
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: 0, obfuscationBypass: IdentityObfuscationProvider()
        ).obfuscate()

        XCTAssertEqual(obfuscationResult.port, .only(5500))
    }

    func testObfuscateShadowsocksPortAutomatic() throws {
        tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(
            state: .shadowsocks,
            shadowsocksPort: .automatic
        )

        let obfuscationResult = RelayObfuscator(
            relays: sampleRelays,
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: 0, obfuscationBypass: IdentityObfuscationProvider()
        ).obfuscate()

        let portRanges = RelaySelector.parseRawPortRanges(sampleRelays.wireguard.shadowsocksPortRanges)

        XCTAssertTrue(
            try portRanges.contains(where: { range in
                range.contains(try XCTUnwrap(obfuscationResult.port.value))
            }))
    }

    func testObfuscateShadowsocksRelayFilteringWithPortOutsideDefaultRanges() throws {
        let allPorts: Range<UInt16> = 1..<65000
        let defaultPortRanges = RelaySelector.parseRawPortRanges(sampleRelays.wireguard.shadowsocksPortRanges)

        let portsOutsideDefaultRange = allPorts.filter { port in
            !defaultPortRanges.contains { range in
                range.contains(port)
            }
        }

        let port = try XCTUnwrap(portsOutsideDefaultRange.randomElement())

        tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(
            state: .shadowsocks,
            shadowsocksPort: .custom(port)
        )

        let obfuscationResult = RelayObfuscator(
            relays: sampleRelays,
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: 0, obfuscationBypass: IdentityObfuscationProvider()
        ).obfuscate()

        let relaysWithExtraAddresses = sampleRelays.wireguard.relays.filter { relay in
            !relay.shadowsocksExtraAddrIn.isNil
        }

        XCTAssertEqual(obfuscationResult.obfuscatedRelays.wireguard.relays.count, relaysWithExtraAddresses.count)
    }

    func testObfuscateShadowsocksRelayFilteringWithPortInsideDefaultRanges() throws {
        let defaultPortRanges = RelaySelector.parseRawPortRanges(sampleRelays.wireguard.shadowsocksPortRanges)
        let port = try XCTUnwrap(defaultPortRanges.randomElement()?.randomElement())

        tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(
            state: .shadowsocks,
            shadowsocksPort: .custom(port)
        )

        let obfuscationResult = RelayObfuscator(
            relays: sampleRelays,
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: 0, obfuscationBypass: IdentityObfuscationProvider()
        ).obfuscate()

        XCTAssertEqual(obfuscationResult.obfuscatedRelays.wireguard.relays.count, sampleRelays.wireguard.relays.count)
    }

    // MARK: QUIC

    func testObfuscateQuicOverPort443() throws {
        tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(
            state: .quic
        )

        let obfuscationResult = RelayObfuscator(
            relays: sampleRelays,
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: 0, obfuscationBypass: IdentityObfuscationProvider()
        ).obfuscate()

        XCTAssertEqual(obfuscationResult.port, defaultQuicPort)
    }

    // MARK: Obfuscation Bypass

    func testObfuscatorBypass() throws {
        tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(state: .automatic)

        let obfuscationResult = RelayObfuscator(
            relays: sampleRelays,
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: 0, obfuscationBypass: ForceShadowsocksObfuscationBypassStub()
        ).obfuscate()

        XCTAssertEqual(obfuscationResult.method, .shadowsocks)
    }
}

struct ForceShadowsocksObfuscationBypassStub: ObfuscationProviding {
    func bypassUnsupportedObfuscation(_: WireGuardObfuscationState) -> WireGuardObfuscationState {
        .shadowsocks
    }
}
