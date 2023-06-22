//
//  TranslucentButtonBlurView.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

class TranslucentButtonBlurView: UIVisualEffectView {
    let appButton: AppButton

    var isEnabled: Bool {
        didSet {
            appButton.isEnabled = isEnabled
            effect = appButton.blurEffect(isEnabled: isEnabled)
        }
    }

    init(button: AppButton) {
        appButton = button
        isEnabled = button.isEnabled

        let effect = appButton.blurEffect(isEnabled: isEnabled)
        super.init(effect: effect)

        contentView.addConstrainedSubviews([button]) {
            button.pinEdgesToSuperview()
        }

        layer.cornerRadius = UIMetrics.controlCornerRadius
        layer.maskedCorners = button.style.cornerMask(effectiveUserInterfaceLayoutDirection)
        layer.masksToBounds = true
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}

private extension AppButton {
    func blurEffect(isEnabled: Bool) -> UIBlurEffect {
        let style = isEnabled ? style.blurEffectStyle : style.disabledStateBlurEffectStyle
        return UIBlurEffect(style: style)
    }
}

private extension AppButton.Style {
    func cornerMask(_ userInterfaceLayoutDirection: UIUserInterfaceLayoutDirection)
        -> CACornerMask {
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

    var disabledStateBlurEffectStyle: UIBlurEffect.Style {
        switch self {
        case .success, .translucentNeutral:
            return .systemThinMaterialDark
        default:
            return blurEffectStyle
        }
    }
}
