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
import UIKit

class VoucherTextField: CustomTextField, UITextFieldDelegate {
    private let inputFormatter = InputTextFormatter(
        configuration: InputTextFormatter.Configuration(
            allowedInput: .alphanumeric(isUpperCase: true),
            groupSeparator: "-",
            groupSize: 4,
            maxGroups: 4
        ))

    private var voucherLength: UInt8 {
        let maxGroups = inputFormatter.configuration.maxGroups
        let groupSize = inputFormatter.configuration.groupSize
        return maxGroups * groupSize + (maxGroups - 1)
    }

    var parsedToken: String {
        inputFormatter.string
    }

    var isVoucherLengthSatisfied: Bool {
        let length = text?.count ?? 0
        return length >= voucherLength
    }

    override init(frame: CGRect) {
        super.init(frame: frame)
        delegate = self
        autocorrectionType = .no
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func canPerformAction(_ action: Selector, withSender sender: Any?) -> Bool {
        if action == #selector(captureTextFromCamera(_:)) { return false }
        return super.canPerformAction(action, withSender: sender)
    }

    // MARK: - UITextFieldDelegate

    func textField(
        _ textField: UITextField,
        shouldChangeCharactersIn range: NSRange,
        replacementString string: String
    ) -> Bool {
        inputFormatter.textField(
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
