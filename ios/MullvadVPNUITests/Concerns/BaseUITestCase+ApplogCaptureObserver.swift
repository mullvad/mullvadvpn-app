//
//  BaseUITestCase+ApplogCaptureObserver.swift
//  MullvadVPN
//
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

@MainActor
extension BaseUITestCase: AppLogConfigurable {
    var target: MullvadExecutableTarget {
        Self.executableTarget
    }
}
