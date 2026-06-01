//
//  SegmentedListItem+Convenience.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-05-04.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

// Convenience inits to make up for the fact that view builders cannot be truly optional.
// Covers most cases and might need to be updated when new combinations of view builder
// params are used.

extension SegmentedListItem where Trailing == EmptyView {
    init(
        level: Int = 0,
        isLastInList: Bool = true,
        isDisabled: Bool = false,
        accessibilityIdentifier: AccessibilityIdentifier? = nil,
        accessibilityLabel: String = "",
        @ViewBuilder leading: @escaping () -> Leading?,
        @ViewBuilder segment: @escaping () -> Segment?,
        @ViewBuilder groupedContent: @escaping () -> GroupedContent?,
        footer: String? = nil,
        onSelect: (() -> Void)? = nil
    ) {
        self.level = level
        self.isLastInList = isLastInList
        self.isDisabled = isDisabled
        self.accessibilityIdentifier = accessibilityIdentifier
        self.accessibilityLabel = accessibilityLabel
        self.leading = leading
        self.trailing = { nil }
        self.segment = segment
        self.groupedContent = groupedContent
        self.footer = footer
        self.onSelect = onSelect
    }
}

extension SegmentedListItem where Trailing == EmptyView, Segment == EmptyView {
    init(
        level: Int = 0,
        isLastInList: Bool = true,
        isDisabled: Bool = false,
        accessibilityIdentifier: AccessibilityIdentifier? = nil,
        accessibilityLabel: String = "",
        @ViewBuilder leading: @escaping () -> Leading?,
        @ViewBuilder groupedContent: @escaping () -> GroupedContent?,
        footer: String? = nil,
        onSelect: (() -> Void)? = nil
    ) {
        self.level = level
        self.isLastInList = isLastInList
        self.isDisabled = isDisabled
        self.accessibilityIdentifier = accessibilityIdentifier
        self.accessibilityLabel = accessibilityLabel
        self.leading = leading
        self.trailing = { nil }
        self.segment = { nil }
        self.groupedContent = groupedContent
        self.footer = footer
        self.onSelect = onSelect
    }
}

extension SegmentedListItem where Trailing == EmptyView, GroupedContent == EmptyView {
    init(
        level: Int = 0,
        isLastInList: Bool = true,
        isDisabled: Bool = false,
        accessibilityIdentifier: AccessibilityIdentifier? = nil,
        accessibilityLabel: String = "",
        @ViewBuilder leading: @escaping () -> Leading?,
        @ViewBuilder segment: @escaping () -> Segment?,
        footer: String? = nil,
        onSelect: (() -> Void)? = nil
    ) {
        self.level = level
        self.isLastInList = isLastInList
        self.isDisabled = isDisabled
        self.accessibilityIdentifier = accessibilityIdentifier
        self.accessibilityLabel = accessibilityLabel
        self.leading = leading
        self.trailing = { nil }
        self.segment = segment
        self.groupedContent = { nil }
        self.footer = footer
        self.onSelect = onSelect
    }
}

extension SegmentedListItem where Trailing == EmptyView, Segment == EmptyView, GroupedContent == EmptyView {
    init(
        level: Int = 0,
        isLastInList: Bool = true,
        isDisabled: Bool = false,
        accessibilityIdentifier: AccessibilityIdentifier? = nil,
        accessibilityLabel: String = "",
        @ViewBuilder leading: @escaping () -> Leading?,
        footer: String? = nil,
        onSelect: (() -> Void)? = nil
    ) {
        self.level = level
        self.isLastInList = isLastInList
        self.isDisabled = isDisabled
        self.accessibilityIdentifier = accessibilityIdentifier
        self.accessibilityLabel = accessibilityLabel
        self.leading = leading
        self.trailing = { nil }
        self.segment = { nil }
        self.groupedContent = { nil }
        self.footer = footer
        self.onSelect = onSelect
    }
}

extension SegmentedListItem where Segment == EmptyView, GroupedContent == EmptyView {
    init(
        level: Int = 0,
        isLastInList: Bool = true,
        isDisabled: Bool = false,
        accessibilityIdentifier: AccessibilityIdentifier? = nil,
        accessibilityLabel: String = "",
        @ViewBuilder leading: @escaping () -> Leading?,
        @ViewBuilder trailing: @escaping () -> Trailing?,
        footer: String? = nil,
        onSelect: (() -> Void)? = nil
    ) {
        self.level = level
        self.isLastInList = isLastInList
        self.isDisabled = isDisabled
        self.accessibilityIdentifier = accessibilityIdentifier
        self.accessibilityLabel = accessibilityLabel
        self.leading = leading
        self.trailing = trailing
        self.segment = { nil }
        self.groupedContent = { nil }
        self.footer = footer
        self.onSelect = onSelect
    }
}
