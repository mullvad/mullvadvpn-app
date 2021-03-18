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
        return CGSize(width: 42, height: 42)
    }

    let primaryButton = AppButton(style: .translucentDangerSplitLeft)
    let secondaryButton = AppButton(style: .translucentDangerSplitRight)

    private let stackView: UIStackView

    init() {
        stackView = UIStackView(arrangedSubviews: [primaryButton, secondaryButton])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .horizontal
        stackView.distribution = .fill
        stackView.alignment = .fill
        stackView.spacing = 1

        primaryButton.titleLabel?.font = UIFont.systemFont(ofSize: 18, weight: .semibold)
        secondaryButton.setImage(UIImage(named: "IconReload"), for: .normal)

        super.init(frame: .zero)

        addSubview(stackView)

        NSLayoutConstraint.activate([
            stackView.leadingAnchor.constraint(equalTo: leadingAnchor),
            stackView.trailingAnchor.constraint(equalTo: trailingAnchor),
            stackView.topAnchor.constraint(equalTo: topAnchor),
            stackView.bottomAnchor.constraint(equalTo: bottomAnchor),

            secondaryButton.widthAnchor.constraint(equalToConstant: self.secondaryButtonSize.width),
            secondaryButton.heightAnchor.constraint(equalToConstant: self.secondaryButtonSize.height)
        ])

        adjustTitleLabelPosition()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func adjustTitleLabelPosition() {
        var contentInsets = AppButton.defaultContentInsets
        contentInsets.left = stackView.spacing + self.secondaryButtonSize.width
        contentInsets.right = 0

        primaryButton.contentEdgeInsets = contentInsets
    }
}
