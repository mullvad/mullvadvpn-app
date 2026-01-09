//
//  RelayObfuscatorTests.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-04.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadMockData
import WireGuardKitTypes
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
        XCTAssertEqual(selectedRelays.exit.endpoint.socketAddress.port, 80)
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

        XCTAssertEqual(selectedRelays.entry?.endpoint.socketAddress.port, 80)
        XCTAssertEqual(selectedRelays.exit.endpoint.socketAddress.port, 5000)
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

// MARK: - IPv6 Filtering Tests

extension RelayObfuscatorTests {
    /// Creates test relays with varying IPv6 support for obfuscation methods.
    /// - Returns: A tuple containing:
    ///   - relays with IPv6 shadowsocks addresses
    ///   - relays with IPv6 QUIC addresses
    ///   - relays with only IPv4 addresses
    ///   - the full server response
    private func createIPv6TestRelays() -> (
        shadowsocksIPv6Relays: [REST.ServerRelay],
        quicIPv6Relays: [REST.ServerRelay],
        ipv4OnlyRelays: [REST.ServerRelay],
        response: REST.ServerRelaysResponse
    ) {
        let shadowsocksIPv6Relay = REST.ServerRelay(
            hostname: "ipv6-shadowsocks-relay",
            active: true,
            owned: true,
            location: "se-sto",
            provider: "Test",
            weight: 100,
            ipv4AddrIn: .loopback,
            ipv6AddrIn: .loopback,
            publicKey: PrivateKey().publicKey.rawValue,
            includeInCountry: true,
            daita: false,
            shadowsocksExtraAddrIn: ["192.168.1.1", "2001:db8::1"], // Has IPv6
            features: nil
        )

        let quicIPv6Relay = REST.ServerRelay(
            hostname: "ipv6-quic-relay",
            active: true,
            owned: true,
            location: "se-sto",
            provider: "Test",
            weight: 100,
            ipv4AddrIn: .loopback,
            ipv6AddrIn: .loopback,
            publicKey: PrivateKey().publicKey.rawValue,
            includeInCountry: true,
            daita: false,
            shadowsocksExtraAddrIn: ["192.168.1.2"], // IPv4 only for shadowsocks
            features: .init(
                daita: nil,
                quic: .init(addrIn: ["192.168.1.2", "2001:db8::2"], domain: "quic.test", token: "token") // Has IPv6
            )
        )

        let bothIPv6Relay = REST.ServerRelay(
            hostname: "ipv6-both-relay",
            active: true,
            owned: true,
            location: "se-sto",
            provider: "Test",
            weight: 100,
            ipv4AddrIn: .loopback,
            ipv6AddrIn: .loopback,
            publicKey: PrivateKey().publicKey.rawValue,
            includeInCountry: true,
            daita: false,
            shadowsocksExtraAddrIn: ["192.168.1.3", "2001:db8::3"], // Has IPv6
            features: .init(
                daita: nil,
                quic: .init(addrIn: ["192.168.1.3", "2001:db8::3"], domain: "quic.test", token: "token") // Has IPv6
            )
        )

        let ipv4OnlyRelay = REST.ServerRelay(
            hostname: "ipv4-only-relay",
            active: true,
            owned: true,
            location: "se-sto",
            provider: "Test",
            weight: 100,
            ipv4AddrIn: .loopback,
            ipv6AddrIn: .loopback,
            publicKey: PrivateKey().publicKey.rawValue,
            includeInCountry: true,
            daita: false,
            shadowsocksExtraAddrIn: ["192.168.1.4"], // IPv4 only
            features: .init(
                daita: nil,
                quic: .init(addrIn: ["192.168.1.4"], domain: "quic.test", token: "token") // IPv4 only
            )
        )

        let noExtraAddrsRelay = REST.ServerRelay(
            hostname: "no-extra-addrs-relay",
            active: true,
            owned: true,
            location: "se-sto",
            provider: "Test",
            weight: 100,
            ipv4AddrIn: .loopback,
            ipv6AddrIn: .loopback,
            publicKey: PrivateKey().publicKey.rawValue,
            includeInCountry: true,
            daita: false,
            shadowsocksExtraAddrIn: nil,
            features: nil
        )

        let allRelays = [shadowsocksIPv6Relay, quicIPv6Relay, bothIPv6Relay, ipv4OnlyRelay, noExtraAddrsRelay]

        let response = REST.ServerRelaysResponse(
            locations: [
                "se-sto": REST.ServerLocation(
                    country: "Sweden",
                    city: "Stockholm",
                    latitude: 59.3289,
                    longitude: 18.0649
                )
            ],
            wireguard: REST.ServerWireguardTunnels(
                ipv4Gateway: .loopback,
                ipv6Gateway: .loopback,
                portRanges: ServerRelaysResponseStubs.wireguardPortRanges,
                relays: allRelays,
                shadowsocksPortRanges: ServerRelaysResponseStubs.shadowsocksPortRanges
            ),
            bridge: REST.ServerBridges(shadowsocks: [], relays: [])
        )

        return (
            shadowsocksIPv6Relays: [shadowsocksIPv6Relay, bothIPv6Relay],
            quicIPv6Relays: [quicIPv6Relay, bothIPv6Relay],
            ipv4OnlyRelays: [ipv4OnlyRelay],
            response: response
        )
    }

    // MARK: - Shadowsocks IPv6 Tests
    // Note: Shadowsocks with IPv6 works with regular ipv6AddrIn for standard ports.
    // Custom ports outside the standard ranges require IPv6 addresses in shadowsocksExtraAddrIn.

    func testShadowsocksWithIPv6AndStandardPortUsesAllRelays() throws {
        let testData = createIPv6TestRelays()

        var settings = LatestTunnelSettings()
        settings.wireGuardObfuscation = WireGuardObfuscationSettings(
            state: .shadowsocks,
            shadowsocksPort: .automatic // Will pick from standard port ranges
        )
        settings.ipVersion = .ipv6

        let obfuscationResult = RelayObfuscator(
            relays: testData.response,
            tunnelSettings: settings,
            connectionAttemptCount: 0,
            obfuscationBypass: IdentityObfuscationProvider()
        ).obfuscate()

        // Standard ports can use regular ipv6AddrIn, so all relays should be available
        XCTAssertEqual(
            obfuscationResult.obfuscatedRelays.wireguard.relays.count,
            testData.response.wireguard.relays.count,
            "Shadowsocks with standard ports should not filter relays for IPv6"
        )
    }

    func testShadowsocksWithIPv6AndCustomPortFiltersToRelaysWithIPv6ExtraAddresses() throws {
        let testData = createIPv6TestRelays()

        let customPort: UInt16 = 12345

        var settings = LatestTunnelSettings()
        settings.wireGuardObfuscation = WireGuardObfuscationSettings(
            state: .shadowsocks,
            shadowsocksPort: .custom(customPort)
        )
        settings.ipVersion = .ipv6

        let obfuscationResult = RelayObfuscator(
            relays: testData.response,
            tunnelSettings: settings,
            connectionAttemptCount: 0,
            obfuscationBypass: IdentityObfuscationProvider()
        ).obfuscate()

        let filteredHostnames = Set(obfuscationResult.obfuscatedRelays.wireguard.relays.map(\.hostname))
        let expectedHostnames = Set(testData.shadowsocksIPv6Relays.map(\.hostname))

        // Custom ports outside standard ranges require IPv6 in shadowsocksExtraAddrIn
        XCTAssertEqual(
            filteredHostnames,
            expectedHostnames,
            "Shadowsocks with custom port and IPv6 should only include relays with IPv6 in shadowsocksExtraAddrIn"
        )
    }

    func testShadowsocksWithIPv4AndCustomPortDoesNotFilterByIPv6() throws {
        let testData = createIPv6TestRelays()

        // Use a custom port outside the standard shadowsocks port ranges
        let customPort: UInt16 = 12345

        var settings = LatestTunnelSettings()
        settings.wireGuardObfuscation = WireGuardObfuscationSettings(
            state: .shadowsocks,
            shadowsocksPort: .custom(customPort)
        )
        settings.ipVersion = .ipv4

        let obfuscationResult = RelayObfuscator(
            relays: testData.response,
            tunnelSettings: settings,
            connectionAttemptCount: 0,
            obfuscationBypass: IdentityObfuscationProvider()
        ).obfuscate()

        // IPv4 mode should filter to relays with shadowsocksExtraAddrIn (for custom port),
        // but not further filter by IPv6
        let relaysWithExtraAddrs = testData.response.wireguard.relays.filter { $0.shadowsocksExtraAddrIn != nil }
        XCTAssertEqual(
            obfuscationResult.obfuscatedRelays.wireguard.relays.count,
            relaysWithExtraAddrs.count,
            "IPv4 mode with custom port should filter by shadowsocksExtraAddrIn presence, not IPv6"
        )
    }

    // MARK: - QUIC IPv6 Tests
    // Note: QUIC requires extra IPv6 addresses - regular entry IPv6 addresses don't work with QUIC.

    func testQuicWithIPv6FiltersToRelaysWithIPv6ExtraAddresses() throws {
        let testData = createIPv6TestRelays()

        var settings = LatestTunnelSettings()
        settings.wireGuardObfuscation = WireGuardObfuscationSettings(state: .quic)
        settings.ipVersion = .ipv6

        let obfuscationResult = RelayObfuscator(
            relays: testData.response,
            tunnelSettings: settings,
            connectionAttemptCount: 0,
            obfuscationBypass: IdentityObfuscationProvider()
        ).obfuscate()

        let filteredHostnames = Set(obfuscationResult.obfuscatedRelays.wireguard.relays.map(\.hostname))
        let expectedHostnames = Set(testData.quicIPv6Relays.map(\.hostname))

        XCTAssertEqual(filteredHostnames, expectedHostnames, "Only relays with IPv6 QUIC addresses should be included")
    }

    func testQuicWithIPv4DoesNotFilterByIPv6() throws {
        let testData = createIPv6TestRelays()

        var settings = LatestTunnelSettings()
        settings.wireGuardObfuscation = WireGuardObfuscationSettings(state: .quic)
        settings.ipVersion = .ipv4

        let obfuscationResult = RelayObfuscator(
            relays: testData.response,
            tunnelSettings: settings,
            connectionAttemptCount: 0,
            obfuscationBypass: IdentityObfuscationProvider()
        ).obfuscate()

        // Should only filter by QUIC support, not IPv6
        let relaysWithQuic = testData.response.wireguard.relays.filter { $0.supportsQuic }
        XCTAssertEqual(
            obfuscationResult.obfuscatedRelays.wireguard.relays.count,
            relaysWithQuic.count,
            "IPv4 mode should only filter by QUIC support, not IPv6 addresses"
        )
    }

    func testQuicWithAutomaticIPVersionDoesNotFilterByIPv6() throws {
        let testData = createIPv6TestRelays()

        var settings = LatestTunnelSettings()
        settings.wireGuardObfuscation = WireGuardObfuscationSettings(state: .quic)
        settings.ipVersion = .automatic

        let obfuscationResult = RelayObfuscator(
            relays: testData.response,
            tunnelSettings: settings,
            connectionAttemptCount: 0,
            obfuscationBypass: IdentityObfuscationProvider()
        ).obfuscate()

        // Should only filter by QUIC support, not IPv6
        let relaysWithQuic = testData.response.wireguard.relays.filter { $0.supportsQuic }
        XCTAssertEqual(
            obfuscationResult.obfuscatedRelays.wireguard.relays.count,
            relaysWithQuic.count,
            "Automatic IP mode should only filter by QUIC support, not IPv6 addresses"
        )
    }

    // MARK: - IPv6 Error Cases

    func testQuicWithIPv6ThrowsErrorWhenNoIPv6RelaysAvailable() throws {
        // Create relays with only IPv4 QUIC addresses
        let ipv4OnlyQuicRelay = REST.ServerRelay(
            hostname: "ipv4-only-quic",
            active: true,
            owned: true,
            location: "se-sto",
            provider: "Test",
            weight: 100,
            ipv4AddrIn: .loopback,
            ipv6AddrIn: .loopback,
            publicKey: PrivateKey().publicKey.rawValue,
            includeInCountry: true,
            daita: false,
            shadowsocksExtraAddrIn: nil,
            features: .init(
                daita: nil,
                quic: .init(addrIn: ["192.168.1.1"], domain: "quic.test", token: "token") // IPv4 only
            )
        )

        let response = REST.ServerRelaysResponse(
            locations: [
                "se-sto": REST.ServerLocation(
                    country: "Sweden",
                    city: "Stockholm",
                    latitude: 59.3289,
                    longitude: 18.0649
                )
            ],
            wireguard: REST.ServerWireguardTunnels(
                ipv4Gateway: .loopback,
                ipv6Gateway: .loopback,
                portRanges: ServerRelaysResponseStubs.wireguardPortRanges,
                relays: [ipv4OnlyQuicRelay],
                shadowsocksPortRanges: ServerRelaysResponseStubs.shadowsocksPortRanges
            ),
            bridge: REST.ServerBridges(shadowsocks: [], relays: [])
        )

        var settings = LatestTunnelSettings()
        settings.wireGuardObfuscation = WireGuardObfuscationSettings(state: .quic)
        settings.ipVersion = .ipv6

        let obfuscationResult = RelayObfuscator(
            relays: response,
            tunnelSettings: settings,
            connectionAttemptCount: 0,
            obfuscationBypass: IdentityObfuscationProvider()
        ).obfuscate()

        // With IPv6 and QUIC, but no IPv6 QUIC addresses, the filtered relays should be empty
        XCTAssertTrue(
            obfuscationResult.obfuscatedRelays.wireguard.relays.isEmpty,
            "QUIC with IPv6 should filter out relays without IPv6 QUIC addresses"
        )
    }

    // MARK: - UDP over TCP (should not filter by IPv6)

    func testUdpOverTcpWithIPv6DoesNotFilterByIPv6Addresses() throws {
        let testData = createIPv6TestRelays()

        var settings = LatestTunnelSettings()
        settings.wireGuardObfuscation = WireGuardObfuscationSettings(
            state: .udpOverTcp,
            udpOverTcpPort: .port80
        )
        settings.ipVersion = .ipv6

        let obfuscationResult = RelayObfuscator(
            relays: testData.response,
            tunnelSettings: settings,
            connectionAttemptCount: 0,
            obfuscationBypass: IdentityObfuscationProvider()
        ).obfuscate()

        // UDP over TCP uses regular IPv6 addresses, so no extra filtering needed
        XCTAssertEqual(
            obfuscationResult.obfuscatedRelays.wireguard.relays.count,
            testData.response.wireguard.relays.count,
            "UDP over TCP should not filter relays based on extra IPv6 addresses"
        )
    }
}
