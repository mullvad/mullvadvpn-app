#[cfg(test)]
mod sigsum_test {
    use crate::relay_list_transparency::validate::validate_relay_list_envelope;
    use crate::relay_list_transparency::{
        RelayListDigest, RelayListEnvelope, Sha256Bytes, validate,
    };
    use sha2::{Digest, Sha256};

    #[test]
    fn test_validate_relay_list_signature() {
        let sig = RelayListEnvelope::parse(RELAY_LIST_SIGNATURE).unwrap();
        let pubkeys = validate::parse_pubkeys(PUBKEYS, ':').unwrap();
        let payload = validate_relay_list_envelope(&sig, &pubkeys).unwrap();
        let digest: Sha256Bytes = Sha256::digest(RELAY_LIST_CONTENT.as_bytes()).into();
        let digest_hex = RelayListDigest::new(digest);
        assert_eq!(payload.digest, digest_hex);
    }

    #[test]
    fn test_invalid_signature_can_parse_unverified_timestamp() {
        let sig =
            RelayListEnvelope::parse(&format!("{RELAY_LIST_SIGNATURE}bad-signature")).unwrap();
        let pubkeys = validate::parse_pubkeys(PUBKEYS, ':').unwrap();
        let err = validate_relay_list_envelope(&sig, &pubkeys).unwrap_err();
        let payload = err.timestamp_parser.parse_without_verification().unwrap();

        let digest: Sha256Bytes = Sha256::digest(RELAY_LIST_CONTENT.as_bytes()).into();
        let digest_hex = RelayListDigest::new(digest);
        assert_eq!(payload.digest, digest_hex);
    }

    #[test]
    fn test_missing_signature_can_parse_unverified_timestamp() {
        // The API will return this if it couldn't generate a valid sigsum signature.
        let no_sig = "{\"digest\":\"8836b8b33efaec686aeb417e2fac35078e286f9383156be9ee2042ca1f14b677\",\"timestamp\":\"2026-03-25T13:13:23+00:00\"}\n\nNO SIGNATURE";

        let sig = RelayListEnvelope::parse(no_sig).unwrap();
        let pubkeys = validate::parse_pubkeys(PUBKEYS, ':').unwrap();
        let err = validate_relay_list_envelope(&sig, &pubkeys).unwrap_err();
        let payload = err.timestamp_parser.parse_without_verification().unwrap();

        let digest: Sha256Bytes = Sha256::digest(RELAY_LIST_CONTENT.as_bytes()).into();
        let digest_hex = RelayListDigest::new(digest);
        assert_eq!(payload.digest, digest_hex);
    }

    #[test]
    fn test_invalid_pubkey_can_parse_unverified_timestamp() {
        let sig = RelayListEnvelope::parse(RELAY_LIST_SIGNATURE).unwrap();
        let pubkeys = validate::parse_pubkeys(PUBKEYS_INVALID, ':').unwrap();
        let err = validate_relay_list_envelope(&sig, &pubkeys).unwrap_err();
        let payload = err.timestamp_parser.parse_without_verification().unwrap();

        let digest: Sha256Bytes = Sha256::digest(RELAY_LIST_CONTENT.as_bytes()).into();
        let digest_hex = RelayListDigest::new(digest);
        assert_eq!(payload.digest, digest_hex);
    }

    static PUBKEYS: &str = "35809994d285fe3dd50d49c384db49519412008c545cb6588c138a86ae4c3284:9e05c843f17ed7225df58fdfd6ddcd65251aa6db4ad8ea63bd2bf0326e30577d";

    static PUBKEYS_INVALID: &str = "11119994d285fe3dd50d49c384db49519412008c545cb6588c138a86ae4c3284:1111c843f17ed7225df58fdfd6ddcd65251aa6db4ad8ea63bd2bf0326e30577d";

    static RELAY_LIST_SIGNATURE: &str = r#"{"digest":"8836b8b33efaec686aeb417e2fac35078e286f9383156be9ee2042ca1f14b677","timestamp":"2026-03-25T13:13:23+00:00"}

version=2
log=44ad38f8226ff9bd27629a41e55df727308d0a1cd8a2c31d3170048ac1dd22a1
leaf=b182cf840e97db6782c7c41f6a549d6732396d6766be5c12b052fcddd1475c40 8e69ed73284f51ca13d9b1f722c543507851607ec901d073196523a24e94db15c6a6e1db6da313b7500f860e3092eb766438ecb06f72c96f57b4d4219800b206

size=34684
root_hash=84700b46cd71b0ad1f3f38a108ded3de4cc4ad56e048345418c3c3d1c09ad7ae
signature=d2c29dff0cceefa3432eaddd7dbad3c6ba46d7ff12ae4c10c614a7a0ea815e1f9ddc25dc7e98ff45c1a24746032577c583112087eed9425225c780c262a0fb0e
cosignature=f6f91669f7955f542718af9edb6a1276d88a0f8822cba9046e549d0b3796f394 1774444419 f910f2d27f30761d730bd33ee8f8542ddf734618fe40dc25a32ea31a21eb023c034c37d65e9431a0802561b750c531a096bd9f85dce8c7a3fcd27e93be488a04
cosignature=81538d254f0a67aa07172826ef46fe0cd04b7425bd634a50bb0a8419d55fa65f 1774444419 c3433bb35d3aef091feb2a47e9ed2af16140d6fc1ef181ca4ba96573a63e1fc2306c6ed8fa292b162f6c887d6fd4c10b2d3e3bc339604f95022ac6823c1ded0d
cosignature=506972ae99f752df639c749ac50a741b80d95f114a35420838ac06107ea9bfe8 1774444419 8d759c9778390e208f7fcd0d5f4ac45b3f0246fb32af3bdd081d3627a95c5b32c08f40280786fcaea8d78c588ca0fdadd51c9624397ad608a9a3a25e704ce50a
cosignature=3573fb7fdab7434cd0dad9fd6460abf420388055db8aaaec177c248f03a990cc 1774444419 0bda8712c9a6e51d082491ebfa487db49b5d479d63b0c01d5f8339d850911b653d08586b7ade11b860cb9ca23d2d14c8fa9c521031f078c8d6bdeacd55e62c0c
cosignature=d59098d686f47ec1d4d1984c8ac0ab97a3d8a65c7ba8cb6422ada0fae927e194 1774444419 2a8f0c4bd6e281748b532cbaec37c4681847b852790b7655890e75b76f06e65683d9822558e80c43fa91445f14ee74ca4e4c776447d6ce762fab0952a4272705
cosignature=f7a8e45707ef65b294a899fd6a8f463b663e843f68c829bc00447171a582de24 1774444419 f2a061368236cbd8dd9c29e3e73bc885819c496e23f68d9fafd35a631638d1d6450397d83408767fa0d73eabdeaddc69fcb476726ed361a1f72bfa533a6ff205
cosignature=ff2f237a707a2d3a6adfb1600d3375cafe4527ba4fcec793e6093b9b1a4bd79a 1774444419 2c27a61780c5dbfadf530d8541e6c6dad0727c09d30fceb965fcd53b17b55f64ffa883902f7742230d8753ad20795bd1aa585d94a2812b4879350e25980c4d06
cosignature=774fafee07d3b0d9399d669676440a6301db9fa8fe2140d7d352418da25144c5 1774444419 8bfbc5e118102f718b6e57156d0728f08352f9e4e03f6d1366515f75fb668d07bfed7940382bb177e23435a6f06ca73015264b73efa90842955bcc84701ef20a
cosignature=0b00d26b3bf3e0ded29f82803abd34c972cb62752305b0e718cf7b8ec1bf99f6 1774444419 aa953a6eaeafbd4e5aa0da74439c4c785f4cfce9053ab9cd632297592ec328e9b9769950ef7b0c3569450e6b8b23ac398eebee4382d2a5bbf905dc0de554e40b
cosignature=768c9aac6ea5ee9b9c75dd862b70dcb693a2cb37c4ae2f15064e34a1ab260b01 1774444419 796dbc5bfcda8135df513a01aad89bcdf139c9ca6a98587c68b906776225bd63c018e5ea1b5b315b23c5a61b48d2f53914a8eff1f83648cecc2444b2fd5e5f02
cosignature=9a0dcddbd96f6d6d404227b5ff23de7a43f25cdde9790af5ace332d347fac49a 1774444419 05ab5af2b891928503ea718e4a1909c6838d05a4cd7f7812e21191f0241888a295eeabefaf799d6aeac236971666c47eae1c3f71146b0d8bd916699d5efd100d
cosignature=6bdf03b285fce48e00ff9b199cb2b77472dcc4a112f067fa5b274929cb9504e3 1774444419 235180ac93961e4fdc7751944a26f064b0eef9377da53e8d8ff279ad67962622fd15f9d512aa175dd488db3b514f58ff737d80da807101b8fbf2237537946a01
cosignature=6d6a78191e59815eff9131941fc3e08dd716a281c8b886dd67880d552e587a23 1774444419 a66d84f604cdb9e8e0341592193adfe78d8103a0c2247df0adffd4fbd7068548621d3c52671648cdd22620c6b0905e3779030944ab53014093fbc735a9870203
cosignature=cd02db1cc0488c28245d7c3ff50b3e214334c067f2571e849425146bb6bd173d 1774444419 eeae8d97370f4edfd04649f4631ec7e0f65c465a839d4737dc9c467b5994122862792c1883440ad5867729cdf810aafe516fa9c1388dbc2c903cbc97eaf8b00b
cosignature=5da2b3803c2f802eed9744b74e3e4a3d31e1e77ed994ef56730d57fa52f698a5 1774444419 7a8849ea198531d621ca3693ccc15008f711eee54efd98faf5558032fcfa4cd1ba09b4f1c096cf0aefec70da99e9dca4a023a9e5c33f4f6cf25da50a81964106
cosignature=0dad9d849c57687454f100d87fa729b54cb69f7af97346613348defcba9c361c 1774444419 0193712ed69ea8b278f2215e4ea86f47864932b7110410d58b5f7537e07a1a409989064af06e467158aef47b24a12cbf98a7254b9d10c2bfb926a27110bc7607
cosignature=0de5858fa7d8770c17cd17decf4d6345120736308786885b3623bcb852324416 1774444419 0597edbe19143139fd5aa7159c8b16ec0dbd63451bd97960a6cbad86b591ac7f83e76d8f2d6e747cbb59c729bcb0462a70a003ab64253c7c24a1d1114f6d100c

leaf_index=34683
node_hash=97848d7aad3859106aa741deb8f8f85bd4948f48a9d008996504521d271244bb
node_hash=f9d1996addfa21b866496c36f2e453ceae66f2edf79f3ea6dd0e061caca79935
node_hash=f7029097dcde1dd74b5899c54c1b301a00121462d8e1c4c80124f1427ba4a938
node_hash=799ccabb31d7586c35b902935e4561a4cca0449a08319d58d798e224b0351dc7
node_hash=43123c01a4d86b32518d9f9c0155090da6bc282ac0c3db489c4f389e539e94a3
node_hash=0eede816620a6c47e2e2978b423dda1bae33f9d6bb4312ed01afae06b1bed8eb
node_hash=dcf4794ac0a94ce66ff0aa98ba0bb85006b485ecfbbf75faa1ee7b70cf847fa3
node_hash=99220c92ca92b67747cd8411dc570e83d7f6c2098a1fa70315e0f6eb75e90fae
node_hash=1e348fd8c2a4fada1c8c914e1cc923dbdbe00b68ff9772a2aaf0511246261e47
node_hash=de044b87840ca91151f7bf64908851698ae4e905e5d9114f2779aa1112c21a0e"#;

    static RELAY_LIST_CONTENT: &str = r#"{"locations":{"ie-dub":{"country":"Ireland","city":"Dublin","latitude":53.35014,"longitude":-6.266155},"de-fra":{"country":"Germany","city":"Frankfurt","latitude":50.110924,"longitude":8.682127},"se-got":{"country":"Sweden","city":"Gothenburg","latitude":57.70887,"longitude":11.97456},"se-mma":{"country":"Sweden","city":"Malmö","latitude":55.607075,"longitude":13.002716},"aa-rsw":{"country":"Relay Software Country","city":"Relay Software city","latitude":0.0,"longitude":0.0},"se-sto":{"country":"Sweden","city":"Stockholm","latitude":59.3289,"longitude":18.0649}},"openvpn":{"relays":[],"ports":[{"port":1194,"protocol":"udp"},{"port":1195,"protocol":"udp"},{"port":1196,"protocol":"udp"},{"port":1197,"protocol":"udp"},{"port":1300,"protocol":"udp"},{"port":1301,"protocol":"udp"},{"port":1302,"protocol":"udp"},{"port":443,"protocol":"tcp"},{"port":80,"protocol":"tcp"}]},"wireguard":{"relays":[{"hostname":"de-fra-wg-001","location":"de-fra","active":true,"owned":false,"provider":"xtom","stboot":true,"ipv4_addr_in":"85.203.53.104","include_in_country":true,"weight":100,"public_key":"9NuXfdBjkHkVy5IdQN+8wMNS7CiFC3n+VRdFsPzgmVM=","ipv6_addr_in":"2a03:1b20:5:3::104","daita":true,"features":{"daita":{},"lwo":{},"quic":{"addr_in":["85.203.53.108","2a03:1b20:5:3::108"],"token":"28234bf5-c4ec-4f28-8975-7d7dc5d537c9","domain":"de-fra-wg-001.relays.stagemole.eu"}}},{"hostname":"ie-dub-wg-001","location":"ie-dub","active":true,"owned":false,"provider":"M247","stboot":true,"ipv4_addr_in":"85.203.53.102","include_in_country":true,"weight":100,"public_key":"PeXs6GHjC4yKfo+1giEN6gGDkae7wTo8hdZFT1kV3Ho=","ipv6_addr_in":"2a03:1b20:5:3::102","features":{"lwo":{},"quic":{"addr_in":["85.203.53.102","2a03:1b20:5:3::102"],"token":"28234bf5-c4ec-4f28-8975-7d7dc5d537c9","domain":"ie-dub-wg-001.relays.stagemole.eu"}}},{"hostname":"se-got-wg-001","location":"se-got","active":true,"owned":true,"provider":"31173","stboot":true,"ipv4_addr_in":"85.203.53.140","include_in_country":true,"weight":100,"public_key":"ZCvlPfPzOf728BIQcSmWzGFuInKK0SdVTyTCZkdrvUk=","ipv6_addr_in":"2a03:1b20:5:3::140","features":{"lwo":{},"quic":{"addr_in":["2a03:1b20:5:3::14ff"],"token":"28234bf5-c4ec-4f28-8975-7d7dc5d537c9","domain":"se-got-wg-001.relays.stagemole.eu"}}},{"hostname":"se-got-wg-002","location":"aa-rsw","active":true,"owned":true,"provider":"RelaySoftwareTeam","stboot":true,"ipv4_addr_in":"85.203.53.145","include_in_country":true,"weight":0,"public_key":"IyER1oEmmuiijmyjI2D4ihrDuButvK4B00h5Z3+0nRM=","ipv6_addr_in":"2a03:1b20:5:3::145","daita":true,"features":{"daita":{},"lwo":{}}},{"hostname":"se-got-wg-003","location":"aa-rsw","active":true,"owned":true,"provider":"RelaySoftwareTeam","stboot":true,"ipv4_addr_in":"85.203.53.147","include_in_country":true,"weight":0,"public_key":"f6A7xEIcAYhpxNgf2KPj76zlaU/ebqYewmmoIHL+ABQ=","ipv6_addr_in":"2a03:1b20:5:3::147","daita":true,"features":{"daita":{},"lwo":{}}},{"hostname":"se-got-wg-004","location":"aa-rsw","active":true,"owned":true,"provider":"RelaySoftwareTeam","stboot":true,"ipv4_addr_in":"85.203.53.231","include_in_country":true,"weight":0,"public_key":"VYtjgSxNzWi9uRaMlMilxjWeuBVQqdTguamP+Fcjj2o=","ipv6_addr_in":"2a03:1b20:5:3::231","daita":true,"features":{"daita":{}}},{"hostname":"se-got-wg-005","location":"se-got","active":false,"owned":true,"provider":"31173","stboot":true,"ipv4_addr_in":"85.203.53.222","include_in_country":true,"weight":100,"public_key":"czbIog4ERYNb8MkMAZmHZ6dC4Eg7tOAjqgJUgxd9Nnk=","ipv6_addr_in":"2a03:1b20:5:3::222","features":{"lwo":{}}},{"hostname":"se-got-wg-881","location":"aa-rsw","active":true,"owned":true,"provider":"RelaySoftwareTeam","stboot":true,"ipv4_addr_in":"45.130.118.209","include_in_country":true,"weight":0,"public_key":"JcpE5Fvoohm7fBo9/xMsXBv0T3BsXixmGmnJrZPqWWQ=","ipv6_addr_in":"2a03:1b20:5:f011::c88f","daita":true,"features":{"daita":{},"quic":{"addr_in":["45.130.118.209"],"token":"28234bf5-c4ec-4f28-8975-7d7dc5d537c9","domain":"se-got-wg-881.relays.stagemole.eu"}}},{"hostname":"se-sto-wg-001","location":"se-sto","active":true,"owned":true,"provider":"Mullvad","stboot":true,"ipv4_addr_in":"85.203.53.81","include_in_country":true,"weight":100,"public_key":"2KS+F8ZAOUSMwygl2CYqkqFhbi3L5u58b3kIpaylaEk=","ipv6_addr_in":"2a03:1b20:5:3::81","features":{"lwo":{},"quic":{"addr_in":["85.203.53.81","2a03:1b20:5:3::81"],"token":"28234bf5-c4ec-4f28-8975-7d7dc5d537c9","domain":"se-sto-wg-001.relays.stagemole.eu"}}}],"port_ranges":[[53,53],[123,123],[4000,33433],[33565,51820],[52001,60000]],"shadowsocks_port_ranges":[[51900,51949]],"ipv4_gateway":"10.64.0.1","ipv6_gateway":"fc00:bbbb:bbbb:bb01::1"},"bridge":{"relays":[{"hostname":"se-got-br-001","location":"se-got","active":true,"owned":true,"provider":"Mullvad","stboot":true,"ipv4_addr_in":"85.203.53.200","include_in_country":true,"weight":100}],"shadowsocks":[{"protocol":"tcp","port":443,"cipher":"aes-256-gcm","password":"mullvad"},{"protocol":"udp","port":1234,"cipher":"aes-256-cfb","password":"mullvad"},{"protocol":"udp","port":1236,"cipher":"aes-256-gcm","password":"mullvad"}]}}"#;
}
