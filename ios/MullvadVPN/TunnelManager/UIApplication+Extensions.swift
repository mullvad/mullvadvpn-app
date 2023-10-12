//
//  UIApplication+Extensions.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2023-10-02.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

#if canImport(UIKit)

import Foundation
import UIKit

public protocol BackgroundTaskProvider {
    func endBackgroundTask(_ identifier: UIBackgroundTaskIdentifier)

    func beginBackgroundTask(
        withName taskName: String?,
        expirationHandler handler: (() -> Void)?
    ) -> UIBackgroundTaskIdentifier
}

extension UIApplication: BackgroundTaskProvider {}

#endif
