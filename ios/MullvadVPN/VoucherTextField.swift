//
//  VoucherTextField.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-08-05.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class VoucherTextField: CustomTextField, UITextFieldDelegate {
    static let voucherLength = 19

    private let inputFormatter = MaskedInputFormatter(
        configuration: MaskedInputFormatter.Configuration(
            allowedInput: .alphanumeric,
            groupSeparator: .dash,
            groupSize: 4,
            maxGroups: 4,
            shouldUseAllCaps: true
        )
    )

    var satisfiesVoucherLengthRequirement: Bool {
        let textLength = text?.count ?? 0

        return textLength >= Self.voucherLength
    }

    override init(frame: CGRect) {
        super.init(frame: frame)

        delegate = self
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func canPerformAction(_ action: Selector, withSender sender: Any?) -> Bool {
        if #available(iOS 15.0, *),
           action == #selector(captureTextFromCamera(_:)) { return false }

        return super.canPerformAction(action, withSender: sender)
    }

    // MARK: - UITextFieldDelegate

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
