//
//  DisconnectSplitButton.swift
//  MullvadVPN
//
//  Created by pronebird on 29/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class DisconnectSplitButton: UIView {
    private var secondaryButtonSize: CGSize {
        UIMetrics.DisconnectSplitButton.secondaryButton
    }

    let primaryButton = AppButton(style: .translucentDangerSplitLeft)
    let secondaryButton = AppButton(style: .translucentDangerSplitRight)

    private let secondaryButtonWidthConstraint: NSLayoutConstraint
    private let secondaryButtonHeightConstraint: NSLayoutConstraint

    private let stackView: UIStackView

    init() {
        let primaryButtonBlurView = TranslucentButtonBlurView(button: primaryButton)
        let secondaryButtonBlurView = TranslucentButtonBlurView(button: secondaryButton)

        stackView = UIStackView(arrangedSubviews: [primaryButtonBlurView, secondaryButtonBlurView])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .horizontal
        stackView.distribution = .fill
        stackView.alignment = .fill
        stackView.spacing = 1

        secondaryButton.setImage(
            UIImage(named: "IconReload")?.imageFlippedForRightToLeftLayoutDirection(),
            for: .normal
        )
        secondaryButtonWidthConstraint = secondaryButton.widthAnchor.constraint(equalToConstant: 0)
        secondaryButtonHeightConstraint = secondaryButton.heightAnchor
            .constraint(equalToConstant: 0)

        super.init(frame: .zero)

        addSubview(stackView)

        NSLayoutConstraint.activate([
            stackView.leadingAnchor.constraint(equalTo: leadingAnchor),
            stackView.trailingAnchor.constraint(equalTo: trailingAnchor),
            stackView.topAnchor.constraint(equalTo: topAnchor),
            stackView.bottomAnchor.constraint(equalTo: bottomAnchor),

            secondaryButtonWidthConstraint,
            secondaryButtonHeightConstraint,
        ])

        updateTraitConstraints()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func updateTraitConstraints() {
        let newSize = secondaryButtonSize
        secondaryButtonWidthConstraint.constant = newSize.width
        secondaryButtonHeightConstraint.constant = newSize.height
        adjustTitleLabelPosition()
    }

    private func adjustTitleLabelPosition() {
        // Instead of setting contentEdgeInsets manually, we update UIButtonConfiguration
        var primaryButtonConfig = primaryButton.configuration ?? UIButton.Configuration.filled()

        let offset = stackView.spacing + secondaryButtonSize.width

        // Create content insets depending on layout direction
        if case .leftToRight = effectiveUserInterfaceLayoutDirection {
            primaryButtonConfig.contentInsets = NSDirectionalEdgeInsets(
                top: primaryButton.defaultContentInsets.top,
                leading: offset,
                bottom: primaryButton.defaultContentInsets.bottom,
                trailing: 0
            )
        } else {
            primaryButtonConfig.contentInsets = NSDirectionalEdgeInsets(
                top: primaryButton.defaultContentInsets.top,
                leading: 0,
                bottom: primaryButton.defaultContentInsets.bottom,
                trailing: offset
            )
        }

        // Apply updated configuration to the primary button
        primaryButton.configuration = primaryButtonConfig
    }
}
