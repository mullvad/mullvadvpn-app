//
//  CustomButton.swift
//  MullvadVPN
//
//  Created by pronebird on 23/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension UIControl.State {
    var customButtonTitleColor: UIColor? {
        switch self {
        case .normal:
            return UIColor.AppButton.normalTitleColor
        case .disabled:
            return UIColor.AppButton.disabledTitleColor
        case .highlighted:
            return UIColor.AppButton.highlightedTitleColor
        default:
            return nil
        }
    }
}

/// A custom `UIButton` subclass that implements additional layouts for the image
class CustomButton: UIButton, Sendable {
    var imageAlignment: NSDirectionalRectEdge = .leading {
        didSet {
            self.configuration?.imagePlacement = imageAlignment
        }
    }

    var inlineImageSpacing: CGFloat = 4 {
        didSet {
            self.configuration?.imagePadding = inlineImageSpacing
        }
    }

    var titleAlignment: UIButton.Configuration.TitleAlignment = .center {
        didSet {
            self.configuration?.titleAlignment = titleAlignment
        }
    }

    var inlineTitleSpacing: CGFloat = 4 {
        didSet {
            self.configuration?.titlePadding = inlineTitleSpacing
        }
    }

    override init(frame: CGRect) {
        super.init(frame: frame)
        commonInit()
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
        commonInit()
    }

    private func commonInit() {
        var config = UIButton.Configuration.plain()
        config.imagePadding = inlineImageSpacing
        config.imagePlacement = imageAlignment
        config.titleAlignment = titleAlignment
        config.titleLineBreakMode = .byWordWrapping
        config.titlePadding = inlineTitleSpacing
        config.baseForegroundColor = state.customButtonTitleColor
        self.configuration = config
    }
}
