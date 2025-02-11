//
//  LocationSectionHeaderView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-01-25.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class LocationSectionHeaderFooterView: UIView, UIContentView {
    var configuration: UIContentConfiguration {
        get {
            actualConfiguration
        } set {
            guard let newConfiguration = newValue as? Configuration,
                  actualConfiguration != newConfiguration else { return }
            actualConfiguration = newConfiguration
            apply(configuration: newConfiguration)
        }
    }

    private var actualConfiguration: Configuration
    private let nameLabel: UILabel = {
        let label = UILabel()
        label.numberOfLines = 0
        label.lineBreakMode = .byWordWrapping
        label.textColor = .primaryTextColor
        label.font = .systemFont(ofSize: 16, weight: .semibold)
        return label
    }()

    private let actionButton: UIButton = {
        let button = UIButton(type: .system)
        button.setImage(UIImage(systemName: "ellipsis"), for: .normal)
        button.tintColor = UIColor(white: 1, alpha: 0.6)
        return button
    }()

    init(configuration: Configuration) {
        self.actualConfiguration = configuration
        super.init(frame: .zero)
        applyAppearance()
        addSubviews()
        apply(configuration: configuration)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func addSubviews() {
        addConstrainedSubviews([nameLabel, actionButton]) {
            nameLabel.pinEdgesToSuperviewMargins(.all().excluding(.trailing))

            actionButton.pinEdgesToSuperview(PinnableEdges([.trailing(8)]))
            actionButton.heightAnchor.constraint(equalTo: heightAnchor)
            actionButton.widthAnchor.constraint(equalTo: actionButton.heightAnchor)

            actionButton.leadingAnchor.constraint(equalTo: nameLabel.trailingAnchor, constant: 16)
        }
    }

    private func apply(configuration: Configuration) {
        let isActionHidden = configuration.primaryAction == nil
        backgroundColor = configuration.style.backgroundColor
        nameLabel.textColor = configuration.style.textColor
        nameLabel.text = configuration.name
        nameLabel.font = configuration.style.font
        actionButton.isHidden = isActionHidden
        actionButton.accessibilityIdentifier = nil
        actualConfiguration.primaryAction.flatMap { action in
            actionButton.setAccessibilityIdentifier(.openCustomListsMenuButton)
            actionButton.addAction(action, for: .touchUpInside)
        }
    }

    private func applyAppearance() {
        let leadingInset = UIMetrics.locationCellLayoutMargins.leading + 6
        directionalLayoutMargins = NSDirectionalEdgeInsets(top: 8, leading: leadingInset, bottom: 8, trailing: 24)
    }
}

extension LocationSectionHeaderFooterView {
    struct Style: Equatable {
        let font: UIFont
        let textColor: UIColor
        let backgroundColor: UIColor
    }

    struct Configuration: UIContentConfiguration, Equatable {
        let name: String
        let style: Style
        var primaryAction: UIAction?

        func makeContentView() -> UIView & UIContentView {
            LocationSectionHeaderFooterView(configuration: self)
        }

        func updated(for state: UIConfigurationState) -> Configuration {
            self
        }
    }
}
