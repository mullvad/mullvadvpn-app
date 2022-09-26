//
//  SettingsDNSTextCell.swift
//  MullvadVPN
//
//  Created by pronebird on 05/10/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class SettingsDNSTextCell: SettingsCell, UITextFieldDelegate {
    var isValidInput = true {
        didSet {
            updateCellAppearance(animated: false)
        }
    }

    let textField = CustomTextField()

    var onTextChange: ((SettingsDNSTextCell) -> Void)?
    var onReturnKey: ((SettingsDNSTextCell) -> Void)?

    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: style, reuseIdentifier: reuseIdentifier)

        textField.translatesAutoresizingMaskIntoConstraints = false
        textField.font = UIFont.systemFont(ofSize: 17)
        textField.backgroundColor = .clear
        textField.textColor = UIColor.TextField.textColor
        textField.textMargins = UIMetrics.settingsCellLayoutMargins
        textField.placeholder = NSLocalizedString(
            "DNS_TEXT_CELL_PLACEHOLDER",
            tableName: "Settings",
            value: "Enter IP",
            comment: ""
        )
        textField.cornerRadius = 0
        textField.keyboardType = .numbersAndPunctuation
        textField.returnKeyType = .done
        textField.autocorrectionType = .no
        textField.smartInsertDeleteType = .no
        textField.smartDashesType = .no
        textField.smartQuotesType = .no
        textField.spellCheckingType = .no
        textField.autocapitalizationType = .none
        textField.delegate = self

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(textDidChange),
            name: UITextField.textDidChangeNotification,
            object: textField
        )

        backgroundView?.backgroundColor = UIColor.TextField.backgroundColor
        contentView.addSubview(textField)

        overrideUserInterfaceStyle = .light

        NSLayoutConstraint.activate([
            textField.topAnchor.constraint(equalTo: contentView.topAnchor),
            textField.leadingAnchor.constraint(equalTo: contentView.leadingAnchor),
            textField.trailingAnchor.constraint(equalTo: contentView.trailingAnchor),
            textField.bottomAnchor.constraint(equalTo: contentView.bottomAnchor),
        ])

        updateCellAppearance(animated: false)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func prepareForReuse() {
        super.prepareForReuse()

        onTextChange = nil
        onReturnKey = nil

        textField.text = ""
        isValidInput = true
    }

    override func setEditing(_ editing: Bool, animated: Bool) {
        super.setEditing(editing, animated: animated)

        updateCellAppearance(animated: animated)
    }

    @objc func textDidChange() {
        onTextChange?(self)
    }

    private func updateCellAppearance(animated: Bool) {
        if animated {
            UIView.animate(withDuration: 0.25) {
                self.updateCellAppearance()
            }
        } else {
            updateCellAppearance()
        }
    }

    private func updateCellAppearance() {
        textField.isEnabled = isEditing

        if isEditing {
            if isValidInput {
                textField.textColor = UIColor.TextField.textColor
            } else {
                textField.textColor = UIColor.TextField.invalidInputTextColor
            }

            backgroundView?.backgroundColor = UIColor.TextField.backgroundColor
        } else {
            textField.textColor = .white

            backgroundView?.backgroundColor = UIColor.SubCell.backgroundColor
        }
    }

    // MARK: - UITextFieldDelegate

    func textFieldShouldReturn(_ textField: UITextField) -> Bool {
        onReturnKey?(self)
        return true
    }

    func textField(
        _ textField: UITextField,
        shouldChangeCharactersIn range: NSRange,
        replacementString string: String
    ) -> Bool {
        let ipv4AddressCharset = CharacterSet.ipv4AddressCharset
        let ipv6AddressCharset = CharacterSet.ipv6AddressCharset

        return [ipv4AddressCharset, ipv6AddressCharset].contains { charset in
            return string.unicodeScalars.allSatisfy { scalar in
                return charset.contains(scalar)
            }
        }
    }
}
