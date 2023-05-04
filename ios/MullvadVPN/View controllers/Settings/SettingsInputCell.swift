//
//  SettingsInputCell.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-05-05.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

private var textFieldSize = CGSize(width: 100, height: 30)

class SettingsInputCell: SelectableSettingsCell {
    let textField = CustomTextField(frame: CGRect(origin: .zero, size: textFieldSize))
    var toolbarDoneButton = UIBarButtonItem()

    var isValidInput = true { didSet {
        updateTextFieldInputValidity()
    }}

    var inputDidChange: ((String) -> Void)?
    var inputWasConfirmed: ((String) -> Void)?

    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: style, reuseIdentifier: reuseIdentifier)

        toolbarDoneButton = UIBarButtonItem(title: "Done", style: .done, target: self, action: #selector(confirmInput))
        accessoryView = textField

        setUpTextField()
        setUpTextFieldToolbar()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func setSelected(_ selected: Bool, animated: Bool) {
        super.setSelected(selected, animated: animated)

        if selected {
            if textField.text?.isEmpty ?? false {
                textField.becomeFirstResponder()
            }
        } else {
            textField.resignFirstResponder()
        }
    }

    func reset() {
        textField.text = nil
        UITextField.SearchTextFieldAppearance.inactive.apply(to: textField)
    }

    func setInput(_ text: String) {
        textField.text = text
        textFieldDidChange(textField)
    }

    @objc func confirmInput() {
        _ = textFieldShouldReturn(textField)
    }

    @objc private func textFieldDidChange(_ textField: UITextField) {
        if let text = textField.text {
            inputDidChange?(text)
            toolbarDoneButton.isEnabled = isValidInput
        }
    }

    private func setUpTextField() {
        textField.borderStyle = .none
        textField.layer.cornerRadius = 4
        textField.font = .preferredFont(forTextStyle: .body)
        textField.textAlignment = .right
        textField.delegate = self
        textField.keyboardType = .numberPad
        textField.returnKeyType = .done
        textField.textMargins = UIEdgeInsets(top: 0, left: 8, bottom: 0, right: 8)
        textField.addTarget(self, action: #selector(textFieldDidChange), for: .editingChanged)

        UITextField.SearchTextFieldAppearance.inactive.apply(to: textField)
    }

    private func setUpTextFieldToolbar() {
        let toolbar = UIToolbar()
        toolbar.items = [
            UIBarButtonItem(barButtonSystemItem: .flexibleSpace, target: self, action: nil),
            toolbarDoneButton,
        ]
        toolbar.sizeToFit()

        textField.inputAccessoryView = toolbar
    }

    private func updateTextFieldInputValidity() {
        if isValidInput {
            textField.textColor = UIColor.TextField.textColor
        } else {
            textField.textColor = UIColor.TextField.invalidInputTextColor
        }
    }
}

extension SettingsInputCell: UITextFieldDelegate {
    func textFieldDidBeginEditing(_ textField: UITextField) {
        inputDidChange?(textField.text ?? "")
        toolbarDoneButton.isEnabled = isValidInput

        UITextField.SearchTextFieldAppearance.active.apply(to: textField)
    }

    func textFieldShouldReturn(_ textField: UITextField) -> Bool {
        guard isValidInput else { return false }

        inputWasConfirmed?(textField.text ?? "")
        textField.resignFirstResponder()

        return true
    }

    func textFieldDidEndEditing(_ textField: UITextField) {
        UITextField.SearchTextFieldAppearance.inactive.apply(to: textField)
    }
}
