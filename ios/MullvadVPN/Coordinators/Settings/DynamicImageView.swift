//
//  DynamicImageView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-06-12.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import UIKit

class DynamicImageView: UIImageView {
    private let baseSize: CGFloat
    private let textStyle: UIFont.TextStyle

    init(image: UIImage?, baseSize: CGFloat = 18.0, textStyle: UIFont.TextStyle = .body) {
        self.baseSize = baseSize
        self.textStyle = textStyle
        super.init(image: image)
        registerForTraitChanges([UITraitPreferredContentSizeCategory.self]) {
            (self: Self, previousTraitCollection: UITraitCollection) in
            self.invalidateIntrinsicContentSize()
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override var intrinsicContentSize: CGSize {
        let scaledSize = UIFontMetrics(forTextStyle: textStyle).scaledValue(for: baseSize)
        return CGSize(width: scaledSize, height: scaledSize)
    }
}
