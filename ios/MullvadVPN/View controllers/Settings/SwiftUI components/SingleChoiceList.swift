//
//  SingleChoiceList.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2024-11-06.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

/**
  A component presenting a vertical list in the Mullvad style for selecting a single item from a list.
  This is parametrised over a value type known as `Value`, which can be any Equatable type. One would typically use an `enum` for this. As the name suggests, this allows one value to be chosen, which it sets a provided binding to.

  The simplest use case for `SingleChoiceList` is to present a list of options, each of which being a simple value without additional data; i.e.,

  ```swift
  SingleChoiceList(
     title: "Colour",
     options: [.red, .green, .blue],
     value: $colour,
     itemDescription: { NSLocalizedString("colour_\($0)") }
  )
  ```

  `SingleChoiceList` also provides support for having a value that takes a user-defined value, and presents a UI for filling this. In this case, the caller needs to provide not only the UI elements but functions for parsing the entered text to a value and unparsing the value to the text field, like so:

  ```swift
 enum TipAmount {
     case none
     case fivePercent
     case tenPercent
     case custom(Int)
  }

  SingleChoiceList(
     title: "Tip",
     options: [.none, .fivePercent, .tenPercent],
     value: $tipAmount,
     parseCustomValue: { Int($0).map { TipAmount.custom($0) },
     formatCustomValue: {
         if case let .custom(t) = $0 { "\(t)" } else { nil }
     },
     customLabel: "Custom",
     customPrompt: "%  ",
     customFieldMode: .numericText
  )

  ```
  */

// swiftlint:disable function_parameter_count

struct SingleChoiceList<Value>: View where Value: Equatable {
    let title: String
    private let options: [OptionSpec]
    var value: Binding<Value>
    @State var initialValue: Value?
    let tableAccessibilityIdentifier: String
    let itemDescription: (Value) -> String
    let customFieldMode: CustomFieldMode
    // a latch to keep the custom field selected through changes of focus until the user taps elsewhere
    @State var customFieldSelected = false

    /// The configuration for the field for a custom value row
    enum CustomFieldMode {
        /// The field is a text field into which any text may be typed
        case freeText
        /// The field is a text field configured for numeric input; i.e., the user will see a numeric keyboard
        case numericText
    }

    // Assumption: there will be only one custom value input per list.
    // This makes sense if it's something like a port; if we ever need to
    // use this with a type with more than one form of custom value, we will
    // need to add some mitigations
    @State var customValueInput = ""
    @FocusState var customValueIsFocused: Bool
    @State var customValueInputIsInvalid = false

    // an individual option being presented in a row
    fileprivate struct OptionSpec: Identifiable {
        enum OptValue {
            // this row consists of a constant item with a fixed Value. It may only be selected as is
            case literal(Value)
            // this row consists of a text field into which the user can enter a custom value, which may yield a valid Value. This has accompanying text, and functions to translate between text field contents and the Value. (The fromValue method only needs to give a non-nil value if its input is a custom value that could have come from this row.)
            case custom(
                label: String,
                prompt: String,
                legend: String?,
                minInputWidth: CGFloat?,
                maxInputLength: Int?,
                toValue: (String) -> Value?,
                fromValue: (Value) -> String?
            )
        }

        let id: Int
        let value: OptValue
    }

    // an internal constructor, building the element from basics
    fileprivate init(
        title: String,
        optionSpecs: [OptionSpec.OptValue],
        value: Binding<Value>,
        tableAccessibilityIdentifier: String?,
        itemDescription: ((Value) -> String)? = nil,
        customFieldMode: CustomFieldMode = .freeText
    ) {
        self.title = title
        self.options = optionSpecs.enumerated().map { OptionSpec(id: $0.offset, value: $0.element) }
        self.value = value
        self.itemDescription = itemDescription ?? { "\($0)" }
        self.tableAccessibilityIdentifier = tableAccessibilityIdentifier ?? "SingleChoiceList"
        self.customFieldMode = customFieldMode
        self.initialValue = value.wrappedValue
    }

    ///  Create a `SingleChoiceList` presenting a choice of several fixed values.
    ///
    /// - Parameters:
    ///   - title: The title of the list, which is typically the name of the item being chosen.
    ///   - options:  A list of `Value`s to be presented.
    ///   - itemDescription: An optional function that, when given a `Value`, returns the string representation to present in the list. If not provided, this will be generated naïvely using string interpolation.
    init(
        title: String,
        options: [Value],
        value: Binding<Value>,
        tableAccessibilityIdentifier: String? = nil,
        itemDescription: ((Value) -> String)? = nil,
        itemAccessibilityIdentifier: ((Value) -> String)? = nil
    ) {
        self.init(
            title: title,
            optionSpecs: options.map { .literal($0) },
            value: value,
            tableAccessibilityIdentifier: tableAccessibilityIdentifier,
            itemDescription: itemDescription
        )
    }

    /// Create a `SingleChoiceList` presenting a choice of several fixed values, plus a row where the user may enter an argument for a custom value.
    ///
    /// - Parameters:
    ///   - title: The title of the list, which is typically the name of the item being chosen.
    ///   - options:  A list of fixed `Value`s to be presented.
    ///   - tableAccessibilityIdentifier: an optional string value for the accessibility identifier of the table element enclosing the list. If not present, it will be "SingleChoiceList"
    ///   - itemDescription: An optional function that, when given a `Value`, returns the string representation to present in the list. If not provided, this will be generated naïvely using string interpolation. This is only used for the non-custom values.
    ///   - parseCustomValue: A function that attempts to parse the text entered into the text field and produce a `Value` (typically the tagged custom value with an argument applied to it). If the text is not valid for a value, it should return `nil`
    ///   - formatCustomValue: A function that, when passed a `Value` containing user-entered custom data, formats that data into a string, which should match what the user would have entered. This function can expect to only be called for the custom value, and should return `nil` in the event of its argument not being a valid custom value.
    ///   - customLabel: The caption to display in the custom row, next to the text field.
    ///   - customPrompt: The text to display, greyed, in the text field when it is empty. This also serves to set the width of the field, and should be right-padded with spaces as appropriate.
    ///   - customLegend: Optional text to display below the custom field, i.e., to explain sensible values
    ///   - customInputWidth: An optional minimum width (in pseudo-pixels) for the custom input field
    ///   - customInputMaxLength: An optional maximum length to which input is truncated
    ///   - customFieldMode: An enumeration that sets the mode of the custom value entry text field. If this is `.numericText`, the data is expected to be a decimal number, and the device will present a numeric keyboard when the field is focussed. If it is `.freeText`,  a standard alphanumeric keyboard will be presented. If not specified, this defaults to `.freeText`.
    init(
        title: String,
        options: [Value],
        value: Binding<Value>,
        tableAccessibilityIdentifier: String? = nil,
        itemDescription: ((Value) -> String)? = nil,
        parseCustomValue: @escaping ((String) -> Value?),
        formatCustomValue: @escaping ((Value) -> String?),
        customLabel: String,
        customPrompt: String,
        customLegend: String? = nil,
        customInputMinWidth: CGFloat? = nil,
        customInputMaxLength: Int? = nil,
        customFieldMode: CustomFieldMode = .freeText
    ) {
        self.init(
            title: title,
            optionSpecs: options.map { .literal($0) } + [.custom(
                label: customLabel,
                prompt: customPrompt,
                legend: customLegend,
                minInputWidth: customInputMinWidth,
                maxInputLength: customInputMaxLength,
                toValue: parseCustomValue,
                fromValue: formatCustomValue
            )],
            value: value,
            tableAccessibilityIdentifier: tableAccessibilityIdentifier,
            itemDescription: itemDescription,
            customFieldMode: customFieldMode
        )
    }

    // Construct a row with arbitrary content and the correct style
    private func row<V: View>(isSelected: Bool, @ViewBuilder items: () -> V) -> some View {
        HStack {
            Image(uiImage: UIImage.tick).opacity(isSelected ? 1.0 : 0.0)
            Spacer().frame(width: UIMetrics.SettingsCell.selectableSettingsCellLeftViewSpacing)

            items()
        }
        .padding(EdgeInsets(UIMetrics.SettingsCell.layoutMargins))
        .background(
            isSelected
                ? Color(UIColor.Cell.Background.selected)
                : Color(UIColor.Cell.Background.indentationLevelOne)
        )
        .foregroundColor(Color(UIColor.Cell.titleTextColor))
    }

    // Construct a literal row for a specific literal value
    private func literalRow(_ item: Value) -> some View {
        row(
            isSelected: value.wrappedValue == item && !customFieldSelected
        ) {
            Text(verbatim: itemDescription(item))
            Spacer()
        }
        .onTapGesture {
            value.wrappedValue = item
            customValueIsFocused = false
            customValueInput = ""
            customFieldSelected = false
        }
    }

    // Construct the one row with a custom input field for a custom value
    // swiftlint:disable function_body_length
    private func customRow(
        label: String,
        prompt: String,
        inputWidth: CGFloat?,
        maxInputLength: Int?,
        toValue: @escaping (String) -> Value?,
        fromValue: @escaping (Value) -> String?
    ) -> some View {
        row(
            isSelected: value.wrappedValue == toValue(customValueInput) || customFieldSelected
        ) {
            Text(label)
            Spacer()
            TextField(
                "value",
                text: $customValueInput,
                prompt: Text(prompt).foregroundColor(
                    customValueIsFocused
                        ? Color(UIColor.TextField.placeholderTextColor)
                        : Color(UIColor.TextField.inactivePlaceholderTextColor)
                )
            )
            .keyboardType(customFieldMode == .numericText ? .numberPad : .default)
            .multilineTextAlignment(
                customFieldMode == .numericText
                    ? .trailing
                    : .leading
            )
            .frame(minWidth: inputWidth, maxWidth: .infinity)
            .fixedSize()
            .padding(4)
            .foregroundColor(
                customValueIsFocused
                    ? customValueInputIsInvalid
                        ? Color(UIColor.TextField.invalidInputTextColor)
                        : Color(UIColor.TextField.textColor)
                    : Color(UIColor.TextField.inactiveTextColor)
            )
            .background(
                customValueIsFocused
                    ? Color(UIColor.TextField.backgroundColor)
                    : Color(UIColor.TextField.inactiveBackgroundColor)
            )
            .cornerRadius(4.0)
            // .border doesn't honour .cornerRadius, so overlaying a RoundedRectangle is necessary
            .overlay(
                RoundedRectangle(cornerRadius: 4.0)
                    .stroke(
                        customValueInputIsInvalid ? Color(UIColor.TextField.invalidInputTextColor) : .clear,
                        lineWidth: 1
                    )
            )
            .focused($customValueIsFocused)
            .onChange(of: customValueInput) { _ in
                if let maxInputLength {
                    if customValueInput.count > maxInputLength {
                        customValueInput = String(customValueInput.prefix(maxInputLength))
                    }
                }
                if let parsedValue = toValue(customValueInput) {
                    value.wrappedValue = parsedValue
                    customValueInputIsInvalid = false
                } else {
                    // this is not a valid value, so we fall back to the
                    // initial value, showing the invalid-value state if
                    // the field is not empty
                    // As `customValueIsFocused` takes a while to propagate, we
                    // only reset the field to the initial value if it was previously
                    // a custom value. Otherwise, blanking this field when the user
                    // has selected another field will cause the user's choice to be
                    // overridden.
                    if let initialValue, fromValue(value.wrappedValue) != nil {
                        value.wrappedValue = initialValue
                    }
                    customValueInputIsInvalid = !customValueInput.isEmpty
                }
            }
            .onAppear {
                if let valueText = fromValue(value.wrappedValue) {
                    customValueInput = valueText
                }
            }
        }
        .onTapGesture {
            customFieldSelected = true
            if let v = toValue(customValueInput) {
                value.wrappedValue = v
            } else {
                customValueIsFocused = true
            }
        }
    }

    // swiftlint:enable function_body_length

    private func subtitleRow(_ text: String) -> some View {
        HStack {
            Text(text)
                .font(.callout)
                .opacity(0.6)
            Spacer()
        }
        .padding(.horizontal, UIMetrics.SettingsCell.layoutMargins.leading)
        .padding(.vertical, 4)
        .background(
            Color(.secondaryColor)
        )
        .foregroundColor(Color(UIColor.Cell.titleTextColor))
    }

    var body: some View {
        VStack(spacing: UIMetrics.TableView.separatorHeight) {
            HStack {
                Text(title).fontWeight(.semibold)
                Spacer()
            }
            .padding(EdgeInsets(UIMetrics.SettingsCell.layoutMargins))
            .background(Color(UIColor.Cell.Background.normal))
            List {
                Section {
                    ForEach(options) { opt in
                        switch opt.value {
                        case let .literal(v):
                            literalRow(v)
                                .listRowSeparator(.hidden)
                        case let .custom(
                            label,
                            prompt,
                            legend,
                            inputWidth,
                            maxInputLength,
                            toValue,
                            fromValue
                        ):
                            customRow(
                                label: label,
                                prompt: prompt,
                                inputWidth: inputWidth,
                                maxInputLength: maxInputLength,
                                toValue: toValue,
                                fromValue: fromValue
                            )
                            .listRowSeparator(.hidden)
                            if let legend {
                                subtitleRow(legend)
                                    .listRowSeparator(.hidden)
                            }
                        }
                    }
                }
                .listRowInsets(.init()) // remove insets
            }
            .accessibilityIdentifier(tableAccessibilityIdentifier)
            .listStyle(.plain)
            .listRowSpacing(UIMetrics.TableView.separatorHeight)
            .environment(\.defaultMinListRowHeight, 0)
            Spacer()
        }
        .padding(EdgeInsets(top: 24, leading: 0, bottom: 0, trailing: 0))
        .background(Color(.secondaryColor))
        .foregroundColor(Color(.primaryTextColor))
        .onAppear {
            initialValue = value.wrappedValue
        }
    }
}

// swiftlint:enable function_parameter_count

#Preview("Static values") {
    StatefulPreviewWrapper(1) { SingleChoiceList(title: "Test", options: [1, 2, 3], value: $0) }
}

#Preview("Optional value") {
    enum ExampleValue: Equatable {
        case two
        case three
        case someNumber(Int)
    }
    return StatefulPreviewWrapper(ExampleValue.two) { value in
        VStack {
            let vs = "Value = \(value.wrappedValue)"
            Text(vs)
            SingleChoiceList(
                title: "Test",
                options: [.two, .three],
                value: value,
                parseCustomValue: { Int($0).flatMap { $0 > 3 ? ExampleValue.someNumber($0) : nil } },
                formatCustomValue: { if case let .someNumber(n) = $0 { "\(n)" } else { nil } },
                customLabel: "Custom",
                customPrompt: "Number",
                customLegend: "The legend goes here",
                customInputMinWidth: 120,
                customInputMaxLength: 6,
                customFieldMode: .numericText
            )
        }
    }
} // swiftlint:disable:this file_length
