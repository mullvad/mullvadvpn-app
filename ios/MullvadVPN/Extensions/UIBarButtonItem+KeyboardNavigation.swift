// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import UIKit

extension UIBarButtonItem {
    enum KeyboardNavigationItemType {
        case previous, next

        fileprivate var localizedTitle: String {
            switch self {
            case .previous:
                return NSLocalizedString("Previous", comment: "")
            case .next:
                return NSLocalizedString("Next", comment: "")
            }
        }

        fileprivate var systemImage: UIImage? {
            switch self {
            case .previous:
                return UIImage(systemName: "chevron.up")
            case .next:
                return UIImage(systemName: "chevron.down")
            }
        }
    }

    convenience init(
        keyboardNavigationItemType: KeyboardNavigationItemType,
        target: Any?,
        action: Selector?
    ) {
        self.init(
            image: keyboardNavigationItemType.systemImage,
            style: .plain,
            target: target,
            action: action
        )

        accessibilityLabel = keyboardNavigationItemType.localizedTitle
    }

    static func makeKeyboardNavigationItems(
        _ configurationBlock: (
            _ prevItem: UIBarButtonItem,
            _ nextItem: UIBarButtonItem
        ) -> Void
    ) -> [UIBarButtonItem] {
        let prevButton = UIBarButtonItem(keyboardNavigationItemType: .previous, target: nil, action: nil)
        let nextButton = UIBarButtonItem(keyboardNavigationItemType: .next, target: nil, action: nil)

        configurationBlock(prevButton, nextButton)

        let spacer = UIBarButtonItem(barButtonSystemItem: .fixedSpace, target: nil, action: nil)
        spacer.width = 8

        return [prevButton, spacer, nextButton]
    }
}
