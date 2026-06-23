//
//  TestObservationConfiguration.swift
//  MullvadVPN
//
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import XCTest

final class TestObservationConfiguration: NSObject {
    override init() {
        XCTestObservationCenter.shared.addTestObserver(AppLogCaptureObserver())
    }
}
