//
//  RelayFilterChipView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-20.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

class RelayFilterChipView: UIView {
    private let titleLabel: UILabel = {
        let label = UILabel()
        label.font = UIFont.preferredFont(forTextStyle: .caption1)
        label.adjustsFontForContentSizeCategory = true
        label.textColor = .white
        return label
    }()

    var didTapButton: (() -> Void)?

    init() {
        super.init(frame: .zero)

        let closeButton = IncreasedHitButton()
        closeButton.setImage(
            UIImage(named: "IconCloseSml")?.withTintColor(.white.withAlphaComponent(0.6)),
            for: .normal
        )
        closeButton.addTarget(self, action: #selector(didTapButton(_:)), for: .touchUpInside)

        let container = UIStackView(arrangedSubviews: [titleLabel, closeButton])
        container.spacing = UIMetrics.FilterView.chipViewLabelSpacing
        container.backgroundColor = .primaryColor
        container.layer.cornerRadius = UIMetrics.FilterView.chipViewCornerRadius
        container.layoutMargins = UIMetrics.FilterView.chipViewLayoutMargins
        container.isLayoutMarginsRelativeArrangement = true

        addConstrainedSubviews([container]) {
            container.pinEdgesToSuperview()
        }
    }

    func setTitle(_ text: String) {
        titleLabel.text = text
    }

    @objc private func didTapButton(_ sender: UIButton) {
        didTapButton?()
    }

    required init?(coder aDecoder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
