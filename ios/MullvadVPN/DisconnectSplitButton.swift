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
    var secondaryButton = AppButton(style: .translucentDangerSplitRight)

    private let stackView: UIStackView

    private var secondaryButtonObserver: NSObjectProtocol?

    init() {
        stackView = UIStackView(arrangedSubviews: [primaryButton, secondaryButton])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .horizontal
        stackView.distribution = .fill
        stackView.alignment = .fill
        stackView.spacing = 1

        primaryButton.titleLabel?.font = UIFont.systemFont(ofSize: 18, weight: .semibold)
        primaryButton.setContentHuggingPriority(.defaultLow, for: .horizontal)
        primaryButton.setContentHuggingPriority(.defaultLow, for: .vertical)
        primaryButton.setContentCompressionResistancePriority(.defaultHigh, for: .horizontal)
        primaryButton.setContentCompressionResistancePriority(.defaultHigh, for: .vertical)

        secondaryButton.setImage(UIImage(named: "IconReload"), for: .normal)
        secondaryButton.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        secondaryButton.setContentHuggingPriority(.defaultHigh, for: .vertical)
        secondaryButton.setContentCompressionResistancePriority(UILayoutPriority(50), for: .horizontal)
        secondaryButton.setContentCompressionResistancePriority(UILayoutPriority(50), for: .vertical)

        super.init(frame: .zero)

        addSubview(stackView)

        secondaryButtonObserver = secondaryButton.observe(\.bounds, options: [.new]) { [weak self] (button, change) in
            self?.adjustTitleLabelPosition()
        }

        NSLayoutConstraint.activate([
            stackView.leadingAnchor.constraint(equalTo: leadingAnchor),
            stackView.trailingAnchor.constraint(equalTo: trailingAnchor),
            stackView.topAnchor.constraint(equalTo: topAnchor),
            stackView.bottomAnchor.constraint(equalTo: bottomAnchor),

            primaryButton.heightAnchor.constraint(equalTo: secondaryButton.heightAnchor),
            secondaryButton.widthAnchor.constraint(equalTo: secondaryButton.heightAnchor)
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func adjustTitleLabelPosition() {
        var contentInsets = AppButton.defaultContentInsets
        contentInsets.left = secondaryButton.frame.width + stackView.spacing
        contentInsets.right = 0

        primaryButton.contentEdgeInsets = contentInsets
    }
}
