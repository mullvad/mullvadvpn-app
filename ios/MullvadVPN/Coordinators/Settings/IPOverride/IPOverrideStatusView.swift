//
//  IPOverrideStatusView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-17.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

class IPOverrideStatusView: UIView {
    private lazy var titleLabel: UILabel = {
        let label = UILabel()
        label.font = .mullvadTinySemiBold
        label.adjustsFontForContentSizeCategory = true
        label.textColor = .white
        return label
    }()

    private lazy var statusIcon: UIImageView = {
        return UIImageView()
    }()

    private lazy var descriptionLabel: UILabel = {
        let label = UILabel()
        label.font = .mullvadMiniSemiBold
        label.adjustsFontForContentSizeCategory = true
        label.textColor = .white.withAlphaComponent(0.6)
        label.numberOfLines = 0
        return label
    }()

    init() {
        super.init(frame: .zero)

        let titleContainerView = UIStackView(arrangedSubviews: [titleLabel, statusIcon, UIView()])
        titleContainerView.spacing = 6

        let contentContainterView = UIStackView(arrangedSubviews: [
            titleContainerView,
            descriptionLabel,
        ])
        contentContainterView.axis = .vertical
        contentContainterView.spacing = 4

        addConstrainedSubviews([contentContainterView]) {
            contentContainterView.pinEdgesToSuperview()
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func setStatus(_ status: IPOverrideStatus) {
        titleLabel.text = status.title.uppercased()
        titleLabel.numberOfLines = 0
        statusIcon.image = status.icon
        descriptionLabel.text = status.description
    }
}
