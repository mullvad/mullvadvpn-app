//
//  String+UnicodeLineSeparator.swift
//  MullvadVPN
//
//  Created by pronebird on 03/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Unicode line separator.
/// Declared on macOS as `NSLineSeparatorCharacter` but not on iOS.
private let unicodeLineSeparator: Character = "\u{2028}"

extension String {
    /// Return a new string with all line seprators `\r\n` or `\n` replaced with unicode line separator.
    ///
    /// `NSAttributedString` treats `\n` as a paragraph separator.
    /// - Returns: a new string with all line separators converted to unicode line separator.
    func withUnicodeLineSeparators() -> String {
        String(map { ch in
            ch.isNewline ? unicodeLineSeparator : ch
        })
    }
}
