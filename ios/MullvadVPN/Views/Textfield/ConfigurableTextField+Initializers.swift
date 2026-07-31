//
//  ConfigurableTextField+Initializers.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-07-23.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import SwiftUI

extension ConfigurableTextField where Leading == EmptyView, Trailing == EmptyView {
    init(
        title: LocalizedStringKey? = "",
        placeholder: LocalizedStringKey,
        foregroundColor: Color = .mullvadTextPrimary,
        text: Binding<String>,
        message: Binding<Message?> = .constant(nil),
        accessibilityIdentifier: AccessibilityIdentifier? = nil,
        borderStyle: Binding<BorderStyle> = .constant(.normal),
        configuration: TextFieldNamespace.Configuration = .init()
    ) {
        self.init(
            title: title,
            placeholder: placeholder,
            foregroundColor: foregroundColor,
            text: text,
            message: message,
            accessibilityIdentifier: accessibilityIdentifier,
            borderStyle: borderStyle,
            configuration: configuration,
            leadingView: { EmptyView() },
            trailingView: { EmptyView() })
    }

}

extension ConfigurableTextField where Trailing == EmptyView {

    init(
        title: LocalizedStringKey? = nil,
        placeholder: LocalizedStringKey,
        foregroundColor: Color = .mullvadTextPrimary,
        text: Binding<String>,
        message: Binding<Message?> = .constant(nil),
        accessibilityIdentifier: AccessibilityIdentifier? = nil,
        borderStyle: Binding<BorderStyle> = .constant(.normal),
        configuration: TextFieldNamespace.Configuration = .init(),
        @ViewBuilder leadingView: () -> Leading
    ) {
        self.init(
            title: title,
            placeholder: placeholder,
            foregroundColor: foregroundColor,
            text: text,
            message: message,
            accessibilityIdentifier: accessibilityIdentifier,
            borderStyle: borderStyle,
            configuration: configuration,
            leadingView: leadingView,
            trailingView: { EmptyView() }
        )
    }
}

extension ConfigurableTextField where Leading == EmptyView {
    init(
        title: LocalizedStringKey? = nil,
        placeholder: LocalizedStringKey,
        foregroundColor: Color = .mullvadTextPrimary,
        text: Binding<String>,
        message: Binding<Message?> = .constant(nil),
        accessibilityIdentifier: AccessibilityIdentifier? = nil,
        borderStyle: Binding<BorderStyle> = .constant(.normal),
        configuration: TextFieldNamespace.Configuration = .init(),
        @ViewBuilder trailingView: () -> Trailing
    ) {
        self.init(
            title: title,
            placeholder: placeholder,
            foregroundColor: foregroundColor,
            text: text,
            message: message,
            accessibilityIdentifier: accessibilityIdentifier,
            borderStyle: borderStyle,
            configuration: configuration,
            leadingView: { EmptyView() },
            trailingView: trailingView
        )
    }
}
