//
//  ChipViewCell.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-20.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

class ChipViewCell: UIView, UIContentView {
    var configuration: UIContentConfiguration {
        didSet {
            set(configuration: configuration)
        }
    }

    private let container = {
        let container = UIView()
        container.backgroundColor = .primaryColor
        container.layer.cornerRadius = UIMetrics.FilterView.chipViewCornerRadius
        container.layoutMargins = UIMetrics.FilterView.chipViewLayoutMargins
        return container
    }()

    private let titleLabel: UILabel = {
        let label = UILabel()
        label.setAccessibilityIdentifier(.relayFilterChipLabel)
        label.adjustsFontForContentSizeCategory = true
        label.translatesAutoresizingMaskIntoConstraints = false
        label.numberOfLines = 1
        label.setContentCompressionResistancePriority(.required, for: .horizontal)
        label.setContentHuggingPriority(.required, for: .horizontal)
        return label
    }()

    private let closeButton: IncreasedHitButton = {
        let button = IncreasedHitButton()
        var buttonConfiguration = UIButton.Configuration.plain()
        buttonConfiguration.image = UIImage.Buttons.closeSmall.withTintColor(.white.withAlphaComponent(0.6))
        buttonConfiguration.contentInsets = .zero
        button.setAccessibilityIdentifier(.relayFilterChipCloseButton)
        button.configuration = buttonConfiguration
        return button
    }()

    private lazy var closeButtonActionHandler: UIAction = {
        return UIAction { [weak self] action in
            guard let self,
                  let chipConfiguration = configuration as? ChipConfiguration,
                  let action = chipConfiguration.didTapButton else {
                return
            }
            action()
        }
    }()

    init(configuration: UIContentConfiguration) {
        self.configuration = configuration
        super.init(frame: .zero)
        addSubviews()
        set(configuration: configuration)
    }

    override init(frame: CGRect) {
        self.configuration = ChipConfiguration(group: .filter, title: "", didTapButton: nil)
        super.init(frame: .zero)
        addSubviews()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func addSubviews() {
        self.setAccessibilityIdentifier(.relayFilterChipView)

        let stackView = UIStackView(arrangedSubviews: [titleLabel, closeButton])
        stackView.spacing = UIMetrics.FilterView.chipViewLabelSpacing

        container.addConstrainedSubviews([stackView]) {
            stackView.pinEdgesToSuperviewMargins()
        }
        addConstrainedSubviews([container]) {
            container.pinEdgesToSuperview()
        }
    }

    private func set(configuration: UIContentConfiguration) {
        guard let chipConfiguration = configuration as? ChipConfiguration else { return }
        container.backgroundColor = chipConfiguration.backgroundColor
        titleLabel.text = chipConfiguration.title
        titleLabel.textColor = chipConfiguration.textColor
        titleLabel.font = chipConfiguration.font
        closeButton.isHidden = chipConfiguration.didTapButton == nil
        titleLabel.accessibilityIdentifier = chipConfiguration.accessibilityId?.asString
        if chipConfiguration.didTapButton != nil {
            closeButton.addAction(closeButtonActionHandler, for: .touchUpInside)
        } else {
            closeButton.removeAction(closeButtonActionHandler, for: .touchUpInside)
        }
    }
}

// Custom content configuration
struct ChipConfiguration: UIContentConfiguration {
    enum Group: Hashable {
        case filter, settings
    }

    var group: Group
    var title: String
    var accessibilityId: AccessibilityIdentifier? = nil
    var textColor: UIColor = .white
    var font = UIFont.preferredFont(forTextStyle: .caption1)
    var backgroundColor: UIColor = .primaryColor
    let didTapButton: (() -> Void)?

    func makeContentView() -> UIView & UIContentView {
        return ChipViewCell(configuration: self)
    }

    func updated(for state: UIConfigurationState) -> ChipConfiguration {
        return self
    }
}
