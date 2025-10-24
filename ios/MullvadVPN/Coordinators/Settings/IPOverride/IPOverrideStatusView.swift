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
        label.numberOfLines = 0
        label.setContentHuggingPriority(.defaultLow, for: .horizontal)
        label.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        return label
    }()

    private lazy var statusIcon: UIImageView = {
        let imageView = UIImageView()
        imageView.contentMode = .scaleAspectFit
        imageView.contentMode = .center
        imageView.setContentHuggingPriority(.required, for: .horizontal)
        imageView.setContentCompressionResistancePriority(.required, for: .horizontal)
        return imageView
    }()

    private lazy var descriptionLabel: UILabel = {
        let label = UILabel()
        label.font = .mullvadMiniSemiBold
        label.adjustsFontForContentSizeCategory = true
        label.textColor = .white.withAlphaComponent(0.6)
        label.numberOfLines = 0
        label.setContentHuggingPriority(.required, for: .vertical)
        label.setContentCompressionResistancePriority(.required, for: .vertical)
        return label
    }()

    init() {
        super.init(frame: .zero)

        let titleContainerView = UIStackView(arrangedSubviews: [titleLabel, statusIcon])
        titleContainerView.spacing = 6
        titleContainerView.distribution = .fill

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
        titleLabel.text = status.title
        titleLabel.numberOfLines = 0
        statusIcon.image = status.icon
        descriptionLabel.text = status.description
    }
}
