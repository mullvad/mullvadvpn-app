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

        primaryButton.translatesAutoresizingMaskIntoConstraints = false
        secondaryButton.translatesAutoresizingMaskIntoConstraints = false

        let secondaryButtonSize = UIMetrics.DisconnectSplitButton.secondaryButton
        let image =
            UIImage(resource: .iconReload) // UIImage(resource: .iconReload).resizableImage(withCapInsets: secondaryButton.configuration?.contentInsets).imageFlippedForRightToLeftLayoutDirection()

        secondaryButtonWidthConstraint = secondaryButton.widthAnchor
            .constraint(equalToConstant: secondaryButtonSize.width)
        secondaryButtonHeightConstraint = secondaryButton.heightAnchor
            .constraint(equalToConstant: secondaryButtonSize.height)

        super.init(frame: .zero)

        addConstrainedSubviews([stackView]) {
            stackView.pinEdgesToSuperview()
        }

        NSLayoutConstraint.activate([
            secondaryButtonWidthConstraint,
            secondaryButtonHeightConstraint,
        ])
        secondaryButton.imageView?.contentMode = .scaleAspectFit
        secondaryButton.imageView?.setNeedsDisplay()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
