public struct StorekitTransaction: Encodable {
    let transaction: String

    public init(transaction: String) {
        self.transaction = transaction
    }
}
