//
//  TunnelStateTests.swift
//  MullvadVPNTests
//
//  Created by Andrew Bulhak on 2024-05-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import PacketTunnelCore
import XCTest

final class TunnelStateTests: XCTestCase {
    let arbitrarySelectedRelay = SelectedRelay(
        endpoint: MullvadEndpoint(
            ipv4Relay: IPv4Endpoint(ip: .any, port: 0),
            ipv4Gateway: .any,
            ipv6Gateway: .any,
            publicKey: Data()
        ),
        hostname: "hostname-goes-here",
        location: Location(country: "country", countryCode: "", city: "city", cityCode: "", latitude: 0, longitude: 0),
        retryAttempts: 0
    )

    // MARK: description

    func testDescription_Connecting_NoRelay() {
        XCTAssertEqual(
            TunnelState.connecting(nil, isPostQuantum: false).description,
            "connecting, fetching relay"
        )

        XCTAssertEqual(
            TunnelState.connecting(nil, isPostQuantum: true).description,
            "connecting (PQ), fetching relay"
        )
    }

    func testDescription_Connecting_WithRelay() {
        XCTAssertEqual(
            TunnelState.connecting(arbitrarySelectedRelay, isPostQuantum: false).description,
            "connecting to hostname-goes-here"
        )

        XCTAssertEqual(
            TunnelState.connecting(arbitrarySelectedRelay, isPostQuantum: true).description,
            "connecting (PQ) to hostname-goes-here"
        )
    }

    func testDescription_Connected() {
        XCTAssertEqual(
            TunnelState.connected(arbitrarySelectedRelay, isPostQuantum: false).description,
            "connected to hostname-goes-here"
        )

        XCTAssertEqual(
            TunnelState.connected(arbitrarySelectedRelay, isPostQuantum: true).description,
            "connected (PQ) to hostname-goes-here"
        )
    }

    // MARK: localizedTitleForSecureLabel

    func testLocalizedTitleForSecureLabel_Connecting() {
        XCTAssertEqual(
            TunnelState.connecting(nil, isPostQuantum: false).localizedTitleForSecureLabel,
            "Creating secure connection"
        )

        XCTAssertEqual(
            TunnelState.connecting(nil, isPostQuantum: true).localizedTitleForSecureLabel,
            "Creating quantum secure connection"
        )
    }

    func testLocalizedTitleForSecureLabel_Reconnecting() {
        XCTAssertEqual(
            TunnelState.reconnecting(arbitrarySelectedRelay, isPostQuantum: false).localizedTitleForSecureLabel,
            "Creating secure connection"
        )

        XCTAssertEqual(
            TunnelState.reconnecting(arbitrarySelectedRelay, isPostQuantum: true).localizedTitleForSecureLabel,
            "Creating quantum secure connection"
        )
    }

    func testLocalizedTitleForSecureLabel_Connected() {
        XCTAssertEqual(
            TunnelState.connected(arbitrarySelectedRelay, isPostQuantum: false).localizedTitleForSecureLabel,
            "Secure connection"
        )

        XCTAssertEqual(
            TunnelState.connected(arbitrarySelectedRelay, isPostQuantum: true).localizedTitleForSecureLabel,
            "Quantum secure connection"
        )
    }

    // MARK: localizedAccessibilityLabel

    func testLocalizedAccessibilityLabel_Connecting() {
        XCTAssertEqual(
            TunnelState.connecting(nil, isPostQuantum: false).localizedAccessibilityLabel,
            "Creating secure connection"
        )

        XCTAssertEqual(
            TunnelState.connecting(nil, isPostQuantum: true).localizedAccessibilityLabel,
            "Creating quantum secure connection"
        )
    }

    func testLocalizedAccessibilityLabel_Reconnecting() {
        XCTAssertEqual(
            TunnelState.reconnecting(arbitrarySelectedRelay, isPostQuantum: false).localizedAccessibilityLabel,
            "Reconnecting to city, country"
        )

        XCTAssertEqual(
            TunnelState.reconnecting(arbitrarySelectedRelay, isPostQuantum: true).localizedAccessibilityLabel,
            "Reconnecting to city, country"
        )
    }

    func testLocalizedAccessibilityLabel_Connected() {
        XCTAssertEqual(
            TunnelState.connected(arbitrarySelectedRelay, isPostQuantum: false).localizedAccessibilityLabel,
            "Secure connection. Connected to city, country"
        )

        XCTAssertEqual(
            TunnelState.connected(arbitrarySelectedRelay, isPostQuantum: true).localizedAccessibilityLabel,
            "Quantum secure connection. Connected to city, country"
        )
    }
}
