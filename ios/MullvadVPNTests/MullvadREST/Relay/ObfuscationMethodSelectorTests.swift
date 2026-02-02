//
//  ObfuscationMethodSelectorTests.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import XCTest

@testable import MullvadREST

class ObfuscationMethodSelectorTests: XCTestCase {
    var tunnelSettings = LatestTunnelSettings()

    func testMethodSelectionIsOff() throws {
        (UInt(0)...10).forEach { attempt in
            tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(state: .off)

            var method = ObfuscationMethodSelector.obfuscationMethodBy(
                connectionAttemptCount: attempt,
                tunnelSettings: tunnelSettings, obfuscationBypass: IdentityObfuscationProvider()
            )
            XCTAssertEqual(method, .off)

            tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(state: .automatic)

            method = ObfuscationMethodSelector.obfuscationMethodBy(
                connectionAttemptCount: attempt,
                tunnelSettings: tunnelSettings, obfuscationBypass: IdentityObfuscationProvider()
            )

            let attemptIndex = Int(attempt) % ObfuscationMethodSelector.obfuscationOrder.count
            if attemptIndex == 0 {
                XCTAssertEqual(method, .off)
            } else {
                XCTAssertNotEqual(method, .off)
            }
        }
    }

    func testMethodSelectionIsShadowsock() throws {
        (UInt(0)...10).forEach { attempt in
            tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(state: .shadowsocks)

            var method = ObfuscationMethodSelector.obfuscationMethodBy(
                connectionAttemptCount: attempt,
                tunnelSettings: tunnelSettings, obfuscationBypass: IdentityObfuscationProvider()
            )
            XCTAssertEqual(method, .shadowsocks)

            tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(state: .automatic)

            method = ObfuscationMethodSelector.obfuscationMethodBy(
                connectionAttemptCount: attempt,
                tunnelSettings: tunnelSettings, obfuscationBypass: IdentityObfuscationProvider()
            )

            let attemptIndex = Int(attempt) % ObfuscationMethodSelector.obfuscationOrder.count
            if attemptIndex == 1 {
                XCTAssertEqual(method, .shadowsocks)
            } else {
                XCTAssertNotEqual(method, .shadowsocks)
            }
        }
    }

    func testMethodSelectionQuic() throws {
        (UInt(0)...10).forEach { attempt in
            tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(state: .quic)

            var method = ObfuscationMethodSelector.obfuscationMethodBy(
                connectionAttemptCount: attempt,
                tunnelSettings: tunnelSettings, obfuscationBypass: IdentityObfuscationProvider()
            )
            XCTAssertEqual(method, .quic)

            tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(state: .automatic)

            method = ObfuscationMethodSelector.obfuscationMethodBy(
                connectionAttemptCount: attempt,
                tunnelSettings: tunnelSettings, obfuscationBypass: IdentityObfuscationProvider()
            )

            let attemptIndex = Int(attempt) % ObfuscationMethodSelector.obfuscationOrder.count
            if attemptIndex == 2 {
                XCTAssertEqual(method, .quic)
            } else {
                XCTAssertNotEqual(method, .quic)
            }
        }
    }

    func testMethodSelectionUdpOverTcp() throws {
        (UInt(0)...10).forEach { attempt in
            tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(state: .udpOverTcp)

            var method = ObfuscationMethodSelector.obfuscationMethodBy(
                connectionAttemptCount: attempt,
                tunnelSettings: tunnelSettings, obfuscationBypass: IdentityObfuscationProvider()
            )
            XCTAssertEqual(method, .udpOverTcp)

            tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(state: .automatic)

            method = ObfuscationMethodSelector.obfuscationMethodBy(
                connectionAttemptCount: attempt,
                tunnelSettings: tunnelSettings, obfuscationBypass: IdentityObfuscationProvider()
            )

            let attemptIndex = Int(attempt) % ObfuscationMethodSelector.obfuscationOrder.count
            if attemptIndex == 3 {
                XCTAssertEqual(method, .udpOverTcp)
            } else {
                XCTAssertNotEqual(method, .udpOverTcp)
            }
        }
    }

    func testMethodSelectionLwo() throws {
        (UInt(0)...10).forEach { attempt in
            tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(state: .lwo)

            var method = ObfuscationMethodSelector.obfuscationMethodBy(
                connectionAttemptCount: attempt,
                tunnelSettings: tunnelSettings, obfuscationBypass: IdentityObfuscationProvider()
            )
            XCTAssertEqual(method, .lwo)

            tunnelSettings.wireGuardObfuscation = WireGuardObfuscationSettings(state: .automatic)

            method = ObfuscationMethodSelector.obfuscationMethodBy(
                connectionAttemptCount: attempt,
                tunnelSettings: tunnelSettings, obfuscationBypass: IdentityObfuscationProvider()
            )

            let attemptIndex = Int(attempt) % ObfuscationMethodSelector.obfuscationOrder.count
            if attemptIndex == 4 {
                XCTAssertEqual(method, .lwo)
            } else {
                XCTAssertNotEqual(method, .lwo)
            }
        }
    }
}
