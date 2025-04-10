//
//  RelayFilterAppliedView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-19.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
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

    private var chips: [ChipConfiguration] = []
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
        let filterChips = createFilterChips(for: filter)
        self.filter = filter
        chips.removeAll(where: { $0.group == .filter })
        chips += filterChips
        chipsView.setChips(chips)
        hideIfNeeded()
    }

    func setDaita(_ enabled: Bool) {
        let chip = ChipConfiguration(
            group: .settings,
            title: NSLocalizedString(
                "RELAY_FILTER_APPLIED_DAITA",
                tableName: "RelayFilter",
                value: "Setting: DAITA",
                comment: ""
            ),
            accessibilityId: .daitaFilterPill,
            didTapButton: nil
        )

        setChip(chip, enabled: enabled)
    }

    func setObfuscation(_ enabled: Bool) {
        let chip = ChipConfiguration(
            group: .settings,
            title: NSLocalizedString(
                "RELAY_FILTER_APPLIED_OBFUSCATION",
                tableName: "RelayFilter",
                value: "Setting: Obfuscation",
                comment: ""
            ),
            accessibilityId: .obfuscationFilterPill,
            didTapButton: nil
        )

        setChip(chip, enabled: enabled)
    }

    // MARK: - Private

    private func setChip(_ chip: ChipConfiguration, enabled: Bool) {
        if enabled {
            if !chips.contains(chip) {
                chips.insert(chip, at: 0)
            }
        } else {
            chips.removeAll { $0 == chip }
        }

        chipsView.setChips(chips)
    }

    private func setUpViews() {
        let dummyView = UIView()
        dummyView.layoutMargins = UIMetrics.FilterView.chipViewLayoutMargins

        let contentContainer = UIStackView(arrangedSubviews: [dummyView, chipsView])
        contentContainer.distribution = .fill

        collectionViewHeightConstraint = chipsView.collectionView.heightAnchor
            .constraint(equalToConstant: 8)
        collectionViewHeightConstraint.isActive = true

        dummyView.addConstrainedSubviews([titleLabel]) {
            titleLabel.pinEdgesToSuperviewMargins()
        }

        addConstrainedSubviews([contentContainer]) {
            contentContainer.pinEdgesToSuperview(PinnableEdges([.top(8), .bottom(8), .leading(4), .trailing(4)]))
        }

        // Add KVO for observing collectionView's contentSize changes
        observeContentSize()
    }

    private func hideIfNeeded() {
        isHidden = chips.isEmpty
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
            Task { @MainActor in
                let height = newSize.height == .zero ? 8 : newSize.height
                collectionViewHeightConstraint.constant = height > 80 ? 80 : height
                layoutIfNeeded() // Update the layout
            }
        }
    }
}
