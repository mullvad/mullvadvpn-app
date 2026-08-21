// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
