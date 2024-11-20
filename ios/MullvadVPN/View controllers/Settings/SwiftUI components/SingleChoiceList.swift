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
 The items can be any Hashable type.
 */

struct SingleChoiceList<Value>: View where Value: Equatable {
    let title: String
    private let options: [OptionSpec]
    var value: Binding<Value>
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
            case custom(label: String, prompt: String, toValue: (String) -> Value?, fromValue: (Value) -> String?)
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
    }

    init(title: String, options: [Value], value: Binding<Value>, itemDescription: ((Value) -> String)? = nil) {
        self.init(
            title: title,
            optionSpecs: options.map { .literal($0) },
            value: value,
            itemDescription: itemDescription
        )
    }

    init(
        title: String,
        options: [Value],
        value: Binding<Value>,
        itemDescription: ((Value) -> String)? = nil,
        parseCustomValue: @escaping ((String) -> Value?),
        formatCustomValue: @escaping ((Value) -> String?),
        customLabel: String,
        customPrompt: String
    ) {
        self.init(
            title: title,
            optionSpecs: options.map { .literal($0) } + [.custom(
                label: customLabel,
                prompt: customPrompt,
                toValue: parseCustomValue,
                fromValue: formatCustomValue
            )],
            value: value,
            itemDescription: itemDescription
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
        row(isSelected: value.wrappedValue == item) {
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
        row(isSelected: value.wrappedValue == toValue(customValue)) {
            Text(label)
            Spacer()
            TextField("value", text: $customValue, prompt: Text(prompt))
                .keyboardType(customFieldMode == .numericText ? .numberPad : .default)
                .fixedSize()
                .padding(4)
                .foregroundColor(.black)
                .background(.white)
                .focused($customValueIsFocused)
                .onChange(of: customValue) { newValue in
                    if let v = toValue(customValue) {
                        value.wrappedValue = v
                    } else if let t = fromValue(value.wrappedValue) {
                        customValue = t
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
                case let .custom(label, prompt, toValue, fromValue):
                    customRow(label: label, prompt: prompt, toValue: toValue, fromValue: fromValue)
                }
            }
            Spacer()
        }
        .padding(EdgeInsets(top: 24, leading: 0, bottom: 0, trailing: 0))
        .background(Color(.secondaryColor))
        .foregroundColor(Color(.primaryTextColor))
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
        customPrompt: "Number"
    )
    }
}
