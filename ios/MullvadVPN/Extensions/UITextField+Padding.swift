//
//  UITextField+Padding.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-05-05.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension UITextField {
    func setLeftPadding(_ amount: CGFloat) {
        leftView = createPaddingView(padding: amount)
        leftViewMode = .always
    }

    func setRightPadding(_ amount: CGFloat) {
        rightView = createPaddingView(padding: amount)
        rightViewMode = .always
    }

    private func createPaddingView(padding: CGFloat) -> UIView {
        return UIView(frame: CGRect(x: 0, y: 0, width: padding, height: frame.size.height))
    }
}
