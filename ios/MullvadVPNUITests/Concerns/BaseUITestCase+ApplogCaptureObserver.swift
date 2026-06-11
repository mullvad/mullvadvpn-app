//
//  BaseUITestCase+ApplogCaptureObserver.swift
//  MullvadVPN
//
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

extension BaseUITestCase: @MainActor AppLogConfigurable {
    var target: MullvadExecutableTarget {
        Self.executableTarget
    }
}
