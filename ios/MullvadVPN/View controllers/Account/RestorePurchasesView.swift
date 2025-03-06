//
//  RestorePurchasesView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-08-15.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

class RestorePurchasesView: UIView {
    var restoreButtonAction: (() -> Void)?
    var infoButtonAction: (() -> Void)?

    private lazy var contentView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [
            restoreButton,
            infoButton,
            UIView(), // Pushes the other views to the left.
        ])
        stackView.spacing = UIMetrics.padding8
        return stackView
    }()

    private lazy var restoreButton: UILabel = {
        let label = UILabel()
        label.setAccessibilityIdentifier(.restorePurchasesButton)
        label.attributedText = makeAttributedString()
        label.isUserInteractionEnabled = true
        label.addGestureRecognizer(UITapGestureRecognizer(target: self, action: #selector(didTapRestoreButton)))
        return label
    }()

    private lazy var infoButton: UIButton = {
        let button = IncreasedHitButton(type: .custom)
        button.isExclusiveTouch = true
        button.setImage(UIImage.Buttons.info, for: .normal)
        button.tintColor = .white
        button.addTarget(self, action: #selector(didTapInfoButton), for: .touchUpInside)
        return button
    }()

    override init(frame: CGRect) {
        super.init(frame: frame)

        addConstrainedSubviews([contentView]) {
            contentView.pinEdgesToSuperview()
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func setButtons(enabled: Bool) {
        restoreButton.isUserInteractionEnabled = enabled
        restoreButton.alpha = enabled ? 1 : 0.5
        infoButton.isEnabled = enabled
    }

    private func makeAttributedString() -> NSAttributedString {
        let text = NSLocalizedString(
            "RESTORE_PURCHASES_BUTTON_TITLE",
            tableName: "Account",
            value: "Restore purchases",
            comment: ""
        )

        return NSAttributedString(string: text, attributes: [
            .font: UIFont.systemFont(ofSize: 13, weight: .semibold),
            .foregroundColor: UIColor.white,
            .underlineStyle: NSUnderlineStyle.single.rawValue,
        ])
    }

    @objc private func didTapRestoreButton() {
        restoreButtonAction?()
    }

    @objc private func didTapInfoButton() {
        infoButtonAction?()
    }
}
