//
//  AccountTextField.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

@IBDesignable class AccountTextField: UITextField {

    private let input = AccountTokenInput()

    override init(frame: CGRect) {
        super.init(frame: frame)
        setup()
    }

    required init?(coder aDecoder: NSCoder) {
        super.init(coder: aDecoder)
        setup()
    }

    private func setup() {
        backgroundColor = UIColor.clear

        delegate = input
        pasteDelegate = input
    }

    var autoformattingText: String {
        set {
            input.replace(with: newValue)
            input.updateTextField(self)
        }
        get {
            input.formattedString
        }
    }

    var parsedToken: String {
        return input.parsedString
    }

    override func textRect(forBounds bounds: CGRect) -> CGRect {
        return bounds.insetBy(dx: 14, dy: 12)
    }

    override func editingRect(forBounds bounds: CGRect) -> CGRect {
        return textRect(forBounds: bounds)
    }

}
