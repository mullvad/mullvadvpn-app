//
//  TranslucentButtonBlurView.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

class TranslucentButtonBlurView: UIVisualEffectView {
    init(button: AppButton) {
        let effect = UIBlurEffect(style: button.style.blurEffectStyle)

        super.init(effect: effect)

        button.translatesAutoresizingMaskIntoConstraints = false

        contentView.addSubview(button)

        NSLayoutConstraint.activate([
            button.topAnchor.constraint(equalTo: contentView.topAnchor),
            button.leadingAnchor.constraint(equalTo: contentView.leadingAnchor),
            button.trailingAnchor.constraint(equalTo: contentView.trailingAnchor),
            button.bottomAnchor.constraint(equalTo: contentView.bottomAnchor),
        ])

        layer.cornerRadius = UIMetrics.controlCornerRadius
        layer.maskedCorners = button.style.cornerMask(effectiveUserInterfaceLayoutDirection)
        layer.masksToBounds = true
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}

private extension AppButton.Style {
    func cornerMask(_ userInterfaceLayoutDirection: UIUserInterfaceLayoutDirection)
        -> CACornerMask
    {
        switch (self, userInterfaceLayoutDirection) {
        case (.translucentDangerSplitLeft, .leftToRight),
             (.translucentDangerSplitRight, .rightToLeft):
            return [.layerMinXMinYCorner, .layerMinXMaxYCorner]
        case (.translucentDangerSplitRight, .leftToRight),
             (.translucentDangerSplitLeft, .rightToLeft):
            return [.layerMaxXMinYCorner, .layerMaxXMaxYCorner]
        default:
            return [
                .layerMinXMinYCorner, .layerMinXMaxYCorner,
                .layerMaxXMinYCorner, .layerMaxXMaxYCorner,
            ]
        }
    }

    var blurEffectStyle: UIBlurEffect.Style {
        switch self {
        case .translucentDangerSplitLeft, .translucentDangerSplitRight, .translucentDanger:
            return .systemUltraThinMaterialDark
        default:
            return .light
        }
    }
}
