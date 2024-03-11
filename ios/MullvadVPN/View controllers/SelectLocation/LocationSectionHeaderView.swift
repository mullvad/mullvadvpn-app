//
//  LocationSectionHeaderView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-01-25.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class LocationSectionHeaderView: UIView, UIContentView {
    var configuration: UIContentConfiguration {
        get {
            actualConfiguration
        } set {
            guard let newConfiguration = newValue as? Configuration,
                  actualConfiguration != newConfiguration else { return }
            let previousConfiguration = actualConfiguration
            actualConfiguration = newConfiguration
            apply(configuration: previousConfiguration)
        }
    }

    private var actualConfiguration: Configuration
    private let nameLabel: UILabel = {
        let label = UILabel()
        label.numberOfLines = 1
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

            actionButton.pinEdgesToSuperviewMargins(PinnableEdges([.trailing(.zero)]))
            actionButton.widthAnchor.constraint(equalToConstant: 24)
            actionButton.heightAnchor.constraint(equalTo: actionButton.widthAnchor, multiplier: 1)
            actionButton.centerYAnchor.constraint(equalTo: self.centerYAnchor)

            actionButton.leadingAnchor.constraint(equalTo: nameLabel.trailingAnchor, constant: 16)
        }
    }

    private func apply(configuration: Configuration) {
        let isActionHidden = (configuration.primaryAction == nil)
        nameLabel.text = configuration.name
        actionButton.isHidden = isActionHidden
        actualConfiguration.primaryAction.flatMap { [weak self] action in
            self?.actionButton.addAction(action, for: .touchUpInside)
        }
    }

    private func applyAppearance() {
        backgroundColor = .primaryColor
        directionalLayoutMargins = NSDirectionalEdgeInsets(top: 8, leading: 16, bottom: 8, trailing: 24)
    }
}

extension LocationSectionHeaderView {
    struct Configuration: UIContentConfiguration, Equatable {
        let name: String

        var primaryAction: UIAction?

        func makeContentView() -> UIView & UIContentView {
            LocationSectionHeaderView(configuration: self)
        }

        func updated(for state: UIConfigurationState) -> Configuration {
            self
        }
    }
}
