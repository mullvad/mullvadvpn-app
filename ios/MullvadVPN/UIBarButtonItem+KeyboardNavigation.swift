//
//  UIBarButtonItem+KeyboardNavigation.swift
//  MullvadVPN
//
//  Created by pronebird on 24/02/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension UIBarButtonItem {

    enum KeyboardNavigationItemType {
        case previous, next

        fileprivate var localizedTitle: String {
            switch self {
            case .previous:
                return NSLocalizedString("Previous", comment: "Keyboard navigation toolbar")
            case .next:
                return NSLocalizedString("Next", comment: "Keyboard navigation toolbar")
            }
        }

        @available(iOS 13, *)
        fileprivate var systemImage: UIImage? {
            switch self {
            case .previous:
                return UIImage(systemName: "chevron.up")
            case .next:
                return UIImage(systemName: "chevron.down")
            }
        }
    }

    convenience init(keyboardNavigationItemType: KeyboardNavigationItemType, target: Any?, action: Selector?) {
        if #available(iOS 13, *) {
            self.init(image: keyboardNavigationItemType.systemImage, style: .plain, target: target, action: action)
        } else {
            self.init(title: keyboardNavigationItemType.localizedTitle, style: .plain, target: target, action: action)
        }
        accessibilityLabel = keyboardNavigationItemType.localizedTitle
    }

    static func makeKeyboardNavigationItems(_ configurationBlock: (_ prevItem: UIBarButtonItem, _ nextItem: UIBarButtonItem) -> Void) -> [UIBarButtonItem] {
        let prevButton = UIBarButtonItem(keyboardNavigationItemType: .previous, target: nil, action: nil)
        let nextButton = UIBarButtonItem(keyboardNavigationItemType: .next, target: nil, action: nil)

        configurationBlock(prevButton, nextButton)

        if #available(iOS 13, *) {
            let spacer = UIBarButtonItem(barButtonSystemItem: .fixedSpace, target: nil, action: nil)
            spacer.width = 8

            return [prevButton, spacer, nextButton]
        } else {
            return [prevButton, nextButton]
        }
    }

}
