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

    override init(frame: CGRect) {
        super.init(frame: .zero)
        commonInit()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func commonInit() {
        let primaryButtonBlurView = TranslucentButtonBlurView(button: primaryButton)
        let secondaryButtonBlurView = TranslucentButtonBlurView(button: secondaryButton)

        let stackView = UIStackView(arrangedSubviews: [primaryButtonBlurView, secondaryButtonBlurView])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .horizontal
        stackView.distribution = .fill
        stackView.alignment = .fill
        stackView.spacing = 1

        let secondaryButtonSize = UIMetrics.DisconnectSplitButton.secondaryButton

        addConstrainedSubviews([stackView]) {
            stackView.pinEdgesToSuperview()

            secondaryButton.widthAnchor.constraint(equalToConstant: secondaryButtonSize.width)
            secondaryButton.heightAnchor.constraint(equalToConstant: secondaryButtonSize.height)
        }

        primaryButton.configuration?.contentInsets.leading += secondaryButtonSize

        // Ideally, we shouldn't need to manually resize the image ourselves.
        // However, since UIButton.Configuration doesn't provide a direct property
        // for controlling image scaling (like imageScaling or contentMode in other contexts),
        // manual resizing has been one approach to ensure the image fits within bounds.
        secondaryButton.configuration?.image = UIImage(resource: .iconReload)
            .resizeImage(targetSize: secondaryButtonSize.deducting(insets: secondaryButton.defaultContentInsets))
            .imageFlippedForRightToLeftLayoutDirection()
    }
}
