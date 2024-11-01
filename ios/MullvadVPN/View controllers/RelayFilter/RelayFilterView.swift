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

    private var chips: [ChipConfiguration] = [] {
        didSet {
            isHidden = chips.isEmpty
        }
    }

    private var chipsView = ChipCollectionView()
    private var collectionViewHeightConstraint: NSLayoutConstraint!
    private var filter: RelayFilter?
    private var contentSizeObservation: NSKeyValueObservation?

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
        self.chips.removeAll(where: { $0.group == .filter })
        let filterChips = createFilterChips(for: filter)
        chips = filterChips + chips
        chipsView.setChips(chips)
    }

    func setDaita(_ enabled: Bool) {
        let text = NSLocalizedString(
            "RELAY_FILTER_APPLIED_DAITA",
            tableName: "RelayFilter",
            value: "Setting: DAITA",
            comment: ""
        )
        if enabled {
            chips.append(ChipConfiguration(group: .settings, title: text))
        } else {
            chips.removeAll(where: { $0.title == text })
        }
        chipsView.setChips(chips)
    }

    // MARK: - Private

    private func setUpViews() {
        let contentContainer = UIStackView(arrangedSubviews: [titleLabel, chipsView])
        contentContainer.distribution = .fill
        contentContainer.spacing = UIMetrics.FilterView.labelSpacing

        collectionViewHeightConstraint = chipsView.collectionView.heightAnchor
            .constraint(equalToConstant: 8.0)
        collectionViewHeightConstraint.isActive = true

        addConstrainedSubviews([contentContainer]) {
            contentContainer.pinEdges(.init([.top(0), .bottom(0)]), to: self)
            contentContainer.pinEdges(.init([.leading(4), .trailing(4)]), to: layoutMarginsGuide)
        }

        // Add KVO for observing collectionView's contentSize changes
        observeContentSize()
    }

    private func createFilterChips(for filter: RelayFilter) -> [ChipConfiguration] {
        var filterChips: [ChipConfiguration] = []

        // Ownership Chip
        if let ownershipChip = createOwnershipChip(for: filter.ownership) {
            filterChips.append(ownershipChip)
        }

        // Providers Chip
        if let providersChip = createProvidersChip(for: filter.providers) {
            filterChips.append(providersChip)
        }

        return filterChips
    }

    private func createOwnershipChip(for ownership: RelayFilter.Ownership) -> ChipConfiguration? {
        switch ownership {
        case .any:
            return nil
        case .owned, .rented:
            let title = NSLocalizedString(
                "RELAY_FILTER_APPLIED_OWNERSHIP",
                tableName: "RelayFilter",
                value: ownership == .owned ? "Owned" : "Rented",
                comment: ""
            )
            return ChipConfiguration(group: .filter, title: title, didTapButton: { [weak self] in
                guard var filter = self?.filter else { return }
                filter.ownership = .any
                self?.didUpdateFilter?(filter)
            })
        }
    }

    private func createProvidersChip(for providers: RelayConstraint<[String]>) -> ChipConfiguration? {
        switch providers {
        case .any:
            return nil
        case let .only(providerList):
            let title = String(
                format: NSLocalizedString(
                    "RELAY_FILTER_APPLIED_PROVIDERS",
                    tableName: "RelayFilter",
                    value: "Providers: %d",
                    comment: ""
                ),
                providerList.count
            )
            return ChipConfiguration(group: .filter, title: title, didTapButton: { [weak self] in
                guard var filter = self?.filter else { return }
                filter.providers = .any
                self?.didUpdateFilter?(filter)
            })
        }
    }

    private func observeContentSize() {
        contentSizeObservation = chipsView.collectionView.observe(\.contentSize, options: [
            .new,
            .old,
        ]) { [weak self] _, change in
            guard let self, let newSize = change.newValue else { return }
            let height = newSize.height == .zero ? 8 : newSize.height
            collectionViewHeightConstraint.constant = height > 80 ? 80 : height
            layoutIfNeeded() // Update the layout
        }
    }
}
