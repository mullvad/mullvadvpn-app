//
//  SettingsInputCell.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-05-05.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

class SettingsInputCell: SelectableSettingsCell {
    let textField = CustomTextField(frame: CGRect(origin: .zero, size: CGSize(width: 100, height: 30)))
    var toolbarDoneButton = UIBarButtonItem()

    var isValidInput: Bool {
        didSet {
            updateTextFieldInputValidity()
        }
    }

    var inputDidChange: ((String) -> Void)?
    var inputWasConfirmed: ((String) -> Void)?

    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        isValidInput = true

        super.init(style: style, reuseIdentifier: reuseIdentifier)

        toolbarDoneButton = UIBarButtonItem(
            title: NSLocalizedString(
                "INPUT_CELL_TOOLBAR_BUTTON_DONE",
                tableName: "Preferences",
                value: "Done",
                comment: ""
            ),
            style: .done,
            target: self,
            action: #selector(confirmInput)
        )

        accessoryView = textField

        setUpTextField()
        setUpTextFieldToolbar()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func prepareForReuse() {
        super.prepareForReuse()

        reset()
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
        textField.textMargins = UIMetrics.SettingsCell.inputCellTextFieldLayoutMargins
        textField.addTarget(self, action: #selector(textFieldDidChange), for: .editingChanged)

        UITextField.SearchTextFieldAppearance.inactive.apply(to: textField)
    }

    private func setUpTextFieldToolbar() {
        let toolbar = UIToolbar(frame: CGRect(x: 0, y: 0, width: UIScreen.main.bounds.width, height: 44))
        toolbar.items = [
            UIBarButtonItem(barButtonSystemItem: .flexibleSpace, target: self, action: nil),
            toolbarDoneButton,
        ]

        toolbar.sizeToFit()

        textField.inputAccessoryView = toolbar
    }

    private func updateTextFieldInputValidity() {
        if isValidInput {
            textField.textColor = textField.isEditing ? .SearchTextField.textColor : .SearchTextField.inactiveTextColor
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
