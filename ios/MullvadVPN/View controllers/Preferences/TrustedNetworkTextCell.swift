//
//  TrustedNetworkTextCell.swift
//  MullvadVPN
//
//  Created by pronebird on 31/03/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

class TrustedNetworkTextCell: SettingsCell, UITextFieldDelegate {
    let textField = CustomTextField()

    var onTextChange: ((TrustedNetworkTextCell) -> Void)?
    var onReturnKey: ((TrustedNetworkTextCell) -> Void)?

    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: style, reuseIdentifier: reuseIdentifier)

        textField.translatesAutoresizingMaskIntoConstraints = false
        textField.font = UIFont.systemFont(ofSize: 17)
        textField.backgroundColor = .clear
        textField.textColor = UIColor.TextField.textColor
        textField.textMargins = UIMetrics.settingsCellLayoutMargins
        textField.placeholder = NSLocalizedString(
            "TRUSTED_NETWORK_CELL_PLACEHOLDER",
            tableName: "Settings",
            value: "Enter network SSID",
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
            textField.textColor = UIColor.TextField.textColor

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
}
