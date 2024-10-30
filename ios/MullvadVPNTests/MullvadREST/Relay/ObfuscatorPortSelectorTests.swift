//
//  ObfuscatorPortSelectorTests.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-04.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes
import XCTest

final class ObfuscatorPortSelectorTests: XCTestCase {
    let defaultWireguardPort: RelayConstraint<UInt16> = .only(56)

    let sampleRelays = ServerRelaysResponseStubs.sampleRelays
    var tunnelSettings = LatestTunnelSettings()

    override func setUp() {
        tunnelSettings.relayConstraints.port = defaultWireguardPort
    }

    func testObfuscateOffDoesNotChangeEndpoint() throws {
        tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(state: .off)

        let obfuscationResult = try ObfuscatorPortSelector(
            relays: sampleRelays
        ).obfuscate(
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: 0
        )

        XCTAssertEqual(obfuscationResult.port, defaultWireguardPort)
    }

    // MARK: UdpOverTcp

    func testObfuscateUdpOverTcpPort80() throws {
        tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(
            state: .udpOverTcp,
            udpOverTcpPort: .port80
        )

        let obfuscationResult = try ObfuscatorPortSelector(
            relays: sampleRelays
        ).obfuscate(
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: 0
        )

        XCTAssertEqual(obfuscationResult.port, .only(80))
    }

    func testObfuscateUdpOverTcpPort5001() throws {
        tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(
            state: .udpOverTcp,
            udpOverTcpPort: .port5001
        )

        let obfuscationResult = try ObfuscatorPortSelector(
            relays: sampleRelays
        ).obfuscate(
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: 0
        )

        XCTAssertEqual(obfuscationResult.port, .only(5001))
    }

    func testObfuscateUpdOverTcpPortAutomaticIsPort80OnEvenRetryAttempts() throws {
        tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(
            state: .udpOverTcp,
            udpOverTcpPort: .automatic
        )

        try Array(0 ... 10).filter { $0.isMultiple(of: 2) }.forEach { attempt in
            let obfuscationResult = try ObfuscatorPortSelector(
                relays: sampleRelays
            ).obfuscate(
                tunnelSettings: tunnelSettings,
                connectionAttemptCount: UInt(attempt)
            )

            XCTAssertEqual(obfuscationResult.port, .only(80))
        }
    }

    func testObfuscateUpdOverTcpPortAutomaticIsPort5001OnOddRetryAttempts() throws {
        tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(
            state: .udpOverTcp,
            udpOverTcpPort: .automatic
        )

        try Array(0 ... 10).filter { !$0.isMultiple(of: 2) }.forEach { attempt in
            let obfuscationResult = try ObfuscatorPortSelector(
                relays: sampleRelays
            ).obfuscate(
                tunnelSettings: tunnelSettings,
                connectionAttemptCount: UInt(attempt)
            )

            XCTAssertEqual(obfuscationResult.port, .only(5001))
        }
    }

    // MARK: Shadowsocks

    func testObfuscateShadowsocksPortCustom() throws {
        tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(
            state: .shadowsocks,
            shadowsocksPort: .custom(5500)
        )

        let obfuscationResult = try ObfuscatorPortSelector(
            relays: sampleRelays
        ).obfuscate(
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: 0
        )

        XCTAssertEqual(obfuscationResult.port, .only(5500))
    }

    func testObfuscateShadowsocksPortAutomatic() throws {
        tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(
            state: .shadowsocks,
            shadowsocksPort: .automatic
        )

        let obfuscationResult = try ObfuscatorPortSelector(
            relays: sampleRelays
        ).obfuscate(
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: 0
        )

        let portRanges = RelaySelector.parseRawPortRanges(sampleRelays.wireguard.shadowsocksPortRanges)

        XCTAssertTrue(portRanges.contains(where: { range in
            range.contains(obfuscationResult.port.value ?? 0)
        }))
    }

    func testObfuscateShadowsocksRelayFilteringWithPortOutsideDefaultRanges() throws {
        let allPorts: Range<UInt16> = 1 ..< 65000
        let defaultPortRanges = RelaySelector.parseRawPortRanges(sampleRelays.wireguard.shadowsocksPortRanges)

        let portsOutsideDefaultRange = allPorts.filter { port in
            !defaultPortRanges.contains { range in
                range.contains(port)
            }
        }

        let port = portsOutsideDefaultRange.randomElement() ?? 0

        tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(
            state: .shadowsocks,
            shadowsocksPort: .custom(port)
        )

        let obfuscationResult = try ObfuscatorPortSelector(
            relays: sampleRelays
        ).obfuscate(
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: 0
        )

        let relaysWithExtraAddresses = sampleRelays.wireguard.relays.filter { relay in
            !relay.shadowsocksExtraAddrIn.isNil
        }

        XCTAssertEqual(obfuscationResult.relays.wireguard.relays.count, relaysWithExtraAddresses.count)
    }

    func testObfuscateShadowsocksRelayFilteringWithPortInsideDefaultRanges() throws {
        let defaultPortRanges = RelaySelector.parseRawPortRanges(sampleRelays.wireguard.shadowsocksPortRanges)
        let port = defaultPortRanges.randomElement()?.randomElement() ?? 0

        tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(
            state: .shadowsocks,
            shadowsocksPort: .custom(port)
        )

        let obfuscationResult = try ObfuscatorPortSelector(
            relays: sampleRelays
        ).obfuscate(
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: 0
        )

        XCTAssertEqual(obfuscationResult.relays.wireguard.relays.count, sampleRelays.wireguard.relays.count)
    }
}
