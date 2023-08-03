//
//  RelayFilterAppliedView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-19.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import UIKit

class RelayFilterView: UIView {
    enum Filter {
        case ownership
        case providers
    }

    private let titleLabel: UILabel = {
        let label = UILabel()

        label.text = NSLocalizedString(
            "RELAY_FILTER_APPLIED_TITLE",
            tableName: "RelayFilter",
            value: "Filtered:",
            comment: ""
        )

        label.font = UIFont.preferredFont(forTextStyle: .caption1)
        label.adjustsFontForContentSizeCategory = true
        label.textColor = .white

        return label
    }()

    private let ownershipView = RelayFilterChipView()
    private let providersView = RelayFilterChipView()
    private var filter: RelayFilter?

    var didUpdateFilter: ((RelayFilter) -> Void)?

    init() {
        super.init(frame: .zero)

        setUpViews()
    }

    required init?(coder aDecoder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func setFilter(_ filter: RelayFilter) {
        self.filter = filter

        ownershipView.isHidden = filter.ownership == .any
        providersView.isHidden = filter.providers == .any

        switch filter.ownership {
        case .any:
            break
        case .owned:
            ownershipView.setTitle(localizedOwnershipText(for: "Owned"))
        case .rented:
            ownershipView.setTitle(localizedOwnershipText(for: "Rented"))
        }

        switch filter.providers {
        case .any:
            providersView.isHidden = true
        case let .only(providers):
            providersView.setTitle(localizedProvidersText(for: providers.count))
        }
    }

    private func setUpViews() {
        ownershipView.didTapButton = { [weak self] in
            guard var filter = self?.filter else { return }

            filter.ownership = .any
            self?.didUpdateFilter?(filter)
        }

        providersView.didTapButton = { [weak self] in
            guard var filter = self?.filter else { return }

            filter.providers = .any
            self?.didUpdateFilter?(filter)
        }

        // Add a dummy view at the end to push content to the left.
        let filterContainer = UIStackView(arrangedSubviews: [ownershipView, providersView, UIView()])
        filterContainer.spacing = UIMetrics.FilterView.interChipViewSpacing

        let contentContainer = UIStackView(arrangedSubviews: [titleLabel, filterContainer])
        contentContainer.spacing = UIMetrics.FilterView.labelSpacing

        addConstrainedSubviews([contentContainer]) {
            contentContainer.pinEdges(.init([.top(0), .bottom(0)]), to: self)
            contentContainer.pinEdges(.init([.leading(0), .trailing(0)]), to: layoutMarginsGuide)
        }
    }

    private func localizedOwnershipText(for string: String) -> String {
        return NSLocalizedString(
            "RELAY_FILTER_APPLIED_OWNERSHIP",
            tableName: "RelayFilter",
            value: string,
            comment: ""
        )
    }

    private func localizedProvidersText(for count: Int) -> String {
        return String(
            format: NSLocalizedString(
                "RELAY_FILTER_APPLIED_PROVIDERS",
                tableName: "RelayFilter",
                value: "Providers: %d",
                comment: ""
            ),
            count
        )
    }
}
