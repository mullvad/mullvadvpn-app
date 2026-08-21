// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import SwiftUI

// Convenience inits to make up for the fact that view builders cannot be truly optional.
// Covers most cases and might need to be updated when new combinations of view builder
// params are used.

extension SegmentedListItem where Trailing == EmptyView {
    init(
        level: Int = 0,
        isLastInList: Bool = true,
        userInteraction: SegmentedListItem.UserInteraction = .enabled,
        accessibilityIdentifier: AccessibilityIdentifier? = nil,
        accessibilityLabel: String = "",
        @ViewBuilder leading: @escaping () -> Leading?,
        @ViewBuilder segment: @escaping () -> Segment?,
        @ViewBuilder groupedContent: @escaping () -> GroupedContent?,
        footer: MullvadInfoView? = nil,
        onSelect: (() -> Void)? = nil
    ) {
        self.level = level
        self.isLastInList = isLastInList
        self.userInteraction = userInteraction
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
        userInteraction: SegmentedListItem.UserInteraction = .enabled,
        accessibilityIdentifier: AccessibilityIdentifier? = nil,
        accessibilityLabel: String = "",
        @ViewBuilder leading: @escaping () -> Leading?,
        @ViewBuilder groupedContent: @escaping () -> GroupedContent?,
        footer: MullvadInfoView? = nil,
        onSelect: (() -> Void)? = nil
    ) {
        self.level = level
        self.isLastInList = isLastInList
        self.userInteraction = userInteraction
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
        userInteraction: SegmentedListItem.UserInteraction = .enabled,
        accessibilityIdentifier: AccessibilityIdentifier? = nil,
        accessibilityLabel: String = "",
        @ViewBuilder leading: @escaping () -> Leading?,
        @ViewBuilder segment: @escaping () -> Segment?,
        footer: MullvadInfoView? = nil,
        onSelect: (() -> Void)? = nil
    ) {
        self.level = level
        self.isLastInList = isLastInList
        self.userInteraction = userInteraction
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
        userInteraction: SegmentedListItem.UserInteraction = .enabled,
        accessibilityIdentifier: AccessibilityIdentifier? = nil,
        accessibilityLabel: String = "",
        @ViewBuilder leading: @escaping () -> Leading?,
        footer: MullvadInfoView? = nil,
        onSelect: (() -> Void)? = nil
    ) {
        self.level = level
        self.isLastInList = isLastInList
        self.userInteraction = userInteraction
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
        userInteraction: SegmentedListItem.UserInteraction = .enabled,
        accessibilityIdentifier: AccessibilityIdentifier? = nil,
        accessibilityLabel: String = "",
        @ViewBuilder leading: @escaping () -> Leading?,
        @ViewBuilder trailing: @escaping () -> Trailing?,
        footer: MullvadInfoView? = nil,
        onSelect: (() -> Void)? = nil
    ) {
        self.level = level
        self.isLastInList = isLastInList
        self.userInteraction = userInteraction
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

extension SegmentedListItem where Segment == EmptyView {
    init(
        level: Int = 0,
        isLastInList: Bool = true,
        userInteraction: SegmentedListItem.UserInteraction = .enabled,
        accessibilityIdentifier: AccessibilityIdentifier? = nil,
        accessibilityLabel: String = "",
        @ViewBuilder leading: @escaping () -> Leading?,
        @ViewBuilder trailing: @escaping () -> Trailing?,
        @ViewBuilder groupedContent: @escaping () -> GroupedContent?,
        footer: MullvadInfoView? = nil,
        onSelect: (() -> Void)? = nil
    ) {
        self.level = level
        self.isLastInList = isLastInList
        self.userInteraction = userInteraction
        self.accessibilityIdentifier = accessibilityIdentifier
        self.accessibilityLabel = accessibilityLabel
        self.leading = leading
        self.trailing = trailing
        self.segment = { nil }
        self.groupedContent = groupedContent
        self.footer = footer
        self.onSelect = onSelect
    }
}
