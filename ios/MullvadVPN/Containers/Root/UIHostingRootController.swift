//
//  UIHostingRootController.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2025-06-09.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import SwiftUI

@MainActor
public final class UIHostingRootController<Content: View>: UIHostingController<Content>, RootContainment {
    let preferredHeaderBarPresentation: HeaderBarPresentation
    let prefersHeaderBarHidden: Bool
    let prefersDeviceInfoBarHidden: Bool

    init(
        preferredHeaderBarPresentation: HeaderBarPresentation =
            HeaderBarPresentation(style: .default, showsDivider: false),
        prefersHeaderBarHidden: Bool = false,
        prefersDeviceInfoBarHidden: Bool = true,
        rootView: Content
    ) {
        self.preferredHeaderBarPresentation = preferredHeaderBarPresentation
        self.prefersHeaderBarHidden = prefersHeaderBarHidden
        self.prefersDeviceInfoBarHidden = prefersDeviceInfoBarHidden
        super.init(rootView: rootView)
    }

    required init?(coder aDecoder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
