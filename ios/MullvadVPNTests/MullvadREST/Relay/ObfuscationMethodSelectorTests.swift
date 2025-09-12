//
//  ObfuscationMethodSelectorTests.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
import MullvadSettings
import XCTest

class ObfuscationMethodSelectorTests: XCTestCase {
    var tunnelSettings = LatestTunnelSettings()

    func testMethodSelectionIsOff() throws {
        (UInt(0) ... 10).forEach { attempt in
            tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(state: .off)

            var method = ObfuscationMethodSelector.obfuscationMethodBy(
                connectionAttemptCount: attempt,
                tunnelSettings: tunnelSettings, obfuscationBias: UnbiasedObfuscationBypass()
            )
            XCTAssertEqual(method, .off)

            tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(state: .automatic)

            method = ObfuscationMethodSelector.obfuscationMethodBy(
                connectionAttemptCount: attempt,
                tunnelSettings: tunnelSettings, obfuscationBias: UnbiasedObfuscationBypass()
            )
            if attempt.isOrdered(nth: 1, forEverySetOf: 4) {
                XCTAssertEqual(method, .off)
            } else {
                XCTAssertNotEqual(method, .off)
            }
        }
    }

    func testMethodSelectionIsShadowsock() throws {
        (UInt(0) ... 10).forEach { attempt in
            tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(state: .shadowsocks)

            var method = ObfuscationMethodSelector.obfuscationMethodBy(
                connectionAttemptCount: attempt,
                tunnelSettings: tunnelSettings, obfuscationBias: UnbiasedObfuscationBypass()
            )
            XCTAssertEqual(method, .shadowsocks)

            tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(state: .automatic)

            method = ObfuscationMethodSelector.obfuscationMethodBy(
                connectionAttemptCount: attempt,
                tunnelSettings: tunnelSettings, obfuscationBias: UnbiasedObfuscationBypass()
            )
            if attempt.isOrdered(nth: 2, forEverySetOf: 4) {
                XCTAssertEqual(method, .shadowsocks)
            } else {
                XCTAssertNotEqual(method, .shadowsocks)
            }
        }
    }

    func testMethodSelectionQuic() throws {
        (UInt(0) ... 10).forEach { attempt in
            tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(state: .quic)

            var method = ObfuscationMethodSelector.obfuscationMethodBy(
                connectionAttemptCount: attempt,
                tunnelSettings: tunnelSettings, obfuscationBias: UnbiasedObfuscationBypass()
            )
            XCTAssertEqual(method, .quic)

            tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(state: .automatic)

            method = ObfuscationMethodSelector.obfuscationMethodBy(
                connectionAttemptCount: attempt,
                tunnelSettings: tunnelSettings, obfuscationBias: UnbiasedObfuscationBypass()
            )
            if attempt.isOrdered(nth: 3, forEverySetOf: 4) {
                XCTAssertEqual(method, .quic)
            } else {
                XCTAssertNotEqual(method, .quic)
            }
        }
    }

    func testMethodSelectionUdpOverTcp() throws {
        (UInt(0) ... 10).forEach { attempt in
            tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(state: .udpOverTcp)

            var method = ObfuscationMethodSelector.obfuscationMethodBy(
                connectionAttemptCount: attempt,
                tunnelSettings: tunnelSettings, obfuscationBias: UnbiasedObfuscationBypass()
            )
            XCTAssertEqual(method, .udpOverTcp)

            tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(state: .automatic)

            method = ObfuscationMethodSelector.obfuscationMethodBy(
                connectionAttemptCount: attempt,
                tunnelSettings: tunnelSettings, obfuscationBias: UnbiasedObfuscationBypass()
            )
            if attempt.isOrdered(nth: 4, forEverySetOf: 4) {
                XCTAssertEqual(method, .udpOverTcp)
            } else {
                XCTAssertNotEqual(method, .udpOverTcp)
            }
        }
    }
}
