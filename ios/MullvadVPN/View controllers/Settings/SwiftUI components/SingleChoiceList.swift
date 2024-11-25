//
//  SingleChoiceList.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2024-11-06.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
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

struct SingleChoiceList<Value>: View where Value: Equatable {
    let title: String
    private let options: [OptionSpec]
    var value: Binding<Value>
    @State var initialValue: Value?
    let itemDescription: (Value) -> String
    let customFieldMode: CustomFieldMode

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
    @State var customValue = ""
    @FocusState var customValueIsFocused: Bool

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
        itemDescription: ((Value) -> String)? = nil,
        customFieldMode: CustomFieldMode = .freeText
    ) {
        self.title = title
        self.options = optionSpecs.enumerated().map { OptionSpec(id: $0.offset, value: $0.element) }
        self.value = value
        self.itemDescription = itemDescription ?? { "\($0)" }
        self.customFieldMode = customFieldMode
        self.initialValue = value.wrappedValue
    }

    ///  Create a `SingleChoiceList` presenting a choice of several fixed values.
    ///
    /// - Parameters:
    ///   - title: The title of the list, which is typically the name of the item being chosen.
    ///   - options:  A list of `Value`s to be presented.
    ///   - itemDescription: An optional function that, when given a `Value`, returns the string representation to present in the list. If not provided, this will be generated naïvely using string interpolation.
    init(title: String, options: [Value], value: Binding<Value>, itemDescription: ((Value) -> String)? = nil) {
        self.init(
            title: title,
            optionSpecs: options.map { .literal($0) },
            value: value,
            itemDescription: itemDescription
        )
    }

    /// Create a `SingleChoiceList` presenting a choice of several fixed values, plus a row where the user may enter an argument for a custom value.
    ///
    /// - Parameters:
    ///   - title: The title of the list, which is typically the name of the item being chosen.
    ///   - options:  A list of fixed `Value`s to be presented.
    ///   - itemDescription: An optional function that, when given a `Value`, returns the string representation to present in the list. If not provided, this will be generated naïvely using string interpolation. This is only used for the non-custom values.
    ///   - parseCustomValue: A function that attempts to parse the text entered into the text field and produce a `Value` (typically the tagged custom value with an argument applied to it). If the text is not valid for a value, it should return `nil`
    ///   - formatCustomValue: A function that, when passed a `Value` containing user-entered custom data, formats that data into a string, which should match what the user would have entered. This function can expect to only be called for the custom value, and should return `nil` in the event of its argument not being a valid custom value.
    ///   - customLabel: The caption to display in the custom row, next to the text field.
    ///   - customPrompt: The text to display, greyed, in the text field when it is empty. This also serves to set the width of the field, and should be right-padded with spaces as appropriate.
    ///   - customLegend: Optional text to display below the custom field, i.e., to explain sensible values
    ///   - customFieldMode: An enumeration that sets the mode of the custom value entry text field. If this is `.numericText`, the data is expected to be a decimal number, and the device will present a numeric keyboard when the field is focussed. If it is `.freeText`,  a standard alphanumeric keyboard will be presented. If not specified, this defaults to `.freeText`.
    init(
        title: String,
        options: [Value],
        value: Binding<Value>,
        itemDescription: ((Value) -> String)? = nil,
        parseCustomValue: @escaping ((String) -> Value?),
        formatCustomValue: @escaping ((Value) -> String?),
        customLabel: String,
        customPrompt: String,
        customLegend: String? = nil,
        customFieldMode: CustomFieldMode = .freeText
    ) {
        self.init(
            title: title,
            optionSpecs: options.map { .literal($0) } + [.custom(
                label: customLabel,
                prompt: customPrompt,
                legend: customLegend,
                toValue: parseCustomValue,
                fromValue: formatCustomValue
            )],
            value: value,
            itemDescription: itemDescription,
            customFieldMode: customFieldMode
        )
    }

    // Construct a row with arbitrary content and the correct style
    private func row<V: View>(isSelected: Bool, @ViewBuilder items: () -> V) -> some View {
        HStack {
            Image(uiImage: UIImage(resource: .iconTick)).opacity(isSelected ? 1.0 : 0.0)
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
            isSelected: value.wrappedValue == item && !customValueIsFocused
        ) {
            Text(verbatim: itemDescription(item))
            Spacer()
        }
        .onTapGesture {
            value.wrappedValue = item
            customValueIsFocused = false
        }
    }

    // Construct the one row with a custom input field for a custom value
    private func customRow(
        label: String,
        prompt: String,
        toValue: @escaping (String) -> Value?,
        fromValue: @escaping (Value) -> String?
    ) -> some View {
        row(
            isSelected: value.wrappedValue == toValue(customValue) || customValueIsFocused
        ) {
            Text(label)
            Spacer()
            TextField("value", text: $customValue, prompt: Text(prompt))
                .keyboardType(customFieldMode == .numericText ? .numberPad : .default)
                .fixedSize()
                .padding(4)
                .foregroundColor(Color(UIColor.TextField.textColor))
                .background(Color(UIColor.TextField.backgroundColor))
                .cornerRadius(4.0)
                .focused($customValueIsFocused)
                .onChange(of: customValue) { newValue in
                    if let parsedValue = toValue(customValue) {
                        value.wrappedValue = parsedValue
                    } else if customValue.isEmpty {
                        // user backspaced over input text; this won't form a
                        // valid value, so we fall back to the initial value
                        // and await their next move
                        if let initialValue {
                            value.wrappedValue = initialValue
                        }
                    } else if let t = fromValue(value.wrappedValue) {
                        customValue = t
                    }
                }
                .onAppear {
                    if let valueText = fromValue(value.wrappedValue) {
                        customValue = valueText
                    }
                }
        }
        .onTapGesture {
            if let v = toValue(customValue) {
                value.wrappedValue = v
            } else {
                customValueIsFocused = true
            }
        }
    }

    private func subtitleRow(_ text: String) -> some View {
        HStack {
            Text(text)
                .font(.callout)
                .opacity(0.6)
            Spacer()
        }
        .padding(.horizontal, UIMetrics.SettingsCell.layoutMargins.leading)
        .padding(.vertical, 4)
    }

    var body: some View {
        VStack(spacing: UIMetrics.TableView.separatorHeight) {
            HStack {
                Text(title).fontWeight(.semibold)
                Spacer()
            }
            .padding(EdgeInsets(UIMetrics.SettingsCell.layoutMargins))
            .background(Color(UIColor.Cell.Background.normal))
            ForEach(options) { opt in
                switch opt.value {
                case let .literal(v):
                    literalRow(v)
                case let .custom(label, prompt, legend, toValue, fromValue):
                    customRow(label: label, prompt: prompt, toValue: toValue, fromValue: fromValue)
                    if let legend {
                        subtitleRow(legend)
                    }
                }
            }
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

#Preview("Static values") {
    StatefulPreviewWrapper(1) { SingleChoiceList(title: "Test", options: [1, 2, 3], value: $0) }
}

#Preview("Optional value") {
    enum ExampleValue: Equatable {
        case two
        case three
        case someNumber(Int)
    }
    return StatefulPreviewWrapper(ExampleValue.two) { SingleChoiceList(
        title: "Test",
        options: [.two, .three],
        value: $0,
        parseCustomValue: { Int($0).map { ExampleValue.someNumber($0) } },
        formatCustomValue: { if case let .someNumber(n) = $0 { "\(n)" } else { nil } },
        customLabel: "Custom",
        customPrompt: "Number",
        customLegend: "The legend goes here"
    )
    }
}
