//
//  AccountPhraseService.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2026-07-31.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

private let phraseCoder = PhraseCoder(digits: 16, words: mnemonic_words)

public func encodeAccountPhrase(_ input: String) -> String? {
    phraseCoder.encode(input: input)
}

public func decodeAccountPhrase(_ input: String) -> String? {
    phraseCoder.decode(input: input)
}
