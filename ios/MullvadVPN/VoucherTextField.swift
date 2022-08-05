//
//  VoucherTextField.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-08-05.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class VoucherTextField: CustomTextField {
    var inputFormatter: InputFormatter = .init(
        allowedInput: .alphanumerical,
        groupSeparator: .dash
    )

    override init(frame: CGRect) {
        super.init(frame: frame)
        delegate = self
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func canPerformAction(_ action: Selector, withSender sender: Any?) -> Bool {
        if #available(iOS 15.0, *) {
            if action == #selector(captureTextFromCamera(_:)) {
                return false
            }
        }
        return super.canPerformAction(action, withSender: sender)
    }
}

// MARK: - UITextFieldDelegate

extension VoucherTextField: UITextFieldDelegate {
    func textField(
        _ textField: UITextField,
        shouldChangeCharactersIn range: NSRange,
        replacementString string: String
    ) -> Bool {
        return inputFormatter.textField(
            textField,
            shouldChangeCharactersIn: range,
            replacementString: string
        )
    }

    func textFieldShouldReturn(_ textField: UITextField) -> Bool {
        textField.resignFirstResponder()
        return true
    }
}
