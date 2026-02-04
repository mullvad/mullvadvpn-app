#[cfg(test)]
mod sigsum_test {
    use crate::sigsum::{parse_signature, validate_data};
    use mullvad_api::{RelayListSignature, Sha256Bytes};
    use sha2::{Digest, Sha256};

    #[test]
    fn test_validate_relay_list_signature() {
        let sig = RelayListSignature::from_server_response(RELAY_LIST_SIGNATURE).unwrap();
        let timestamp = parse_signature(&sig).unwrap();
        let digest: Sha256Bytes = Sha256::digest(RELAY_LIST_CONTENT.as_bytes()).into();
        validate_data(&timestamp, &digest).unwrap();
    }

    #[test]
    fn test_invalid_signature_can_parse_unverified_timestamp() {
        let sig = RelayListSignature::from_server_response(&format!(
            "{RELAY_LIST_SIGNATURE}bad-signature"
        ))
        .unwrap();
        let err = parse_signature(&sig).unwrap_err();
        let timestamp = err.parser.parse_timestamp_without_verification().unwrap();

        let digest: Sha256Bytes = Sha256::digest(RELAY_LIST_CONTENT.as_bytes()).into();
        validate_data(&timestamp, &digest).unwrap();
    }

    static RELAY_LIST_SIGNATURE: &str = r#"{"digest":"446fc8ebccd95d5fc07362109b5a4f5eac8c38886ebd6d918d007b6a4fb72865","timestamp":"2026-02-03T10:47:14+00:00"}

version=2
log=1643169b32bef33a3f54f8a353b87c475d19b6223cbb106390d10a29978e1cba
leaf=e3d850008decd4aca3e538a884c8cb5ddb46151c3ff6306f0254a720416401be 553b586360393951302430dba5e008f0230550ca172c7831ef2c04d8ef771710cb1b9dc432a7f5158cf0823c5208f4aa8629372338d5ae85e4886540044eca0f

size=381378
root_hash=929144e5356821b52096ba09c8c37e62b99128025a774f69b2773a96e687a21e
signature=7265e3073f16794b2ebbabda52a4a0db58b9d85d35064180695aef40e81c2b42ba726293998d5094280e2944711e3511343f588d1d86df2caea123c768086508
cosignature=70b861a010f25030de6ff6a5267e0b951e70c04b20ba4a3ce41e7fba7b9b7dfc 1770115653 fc08f7a963682c7756fdf7f90cc87c47e4dbcaeec94224eab5167e051836c7d3b50f764c383b08d752c6844ce90377de32175b34c9ec71589f700bb72a77fd00
cosignature=c1d2d6935c2fb43bef395792b1f3c1dfe4072d4c6cadd05e0cc90b28d7141ed3 1770115653 1e1f650a3085acfee32c2e9d6d50b3c55a754cc4a02de9b47a11798891af887717a05b58d106dd659a9c9f263c01fd2b627b8ab2eba3f68d9d29662a1681b004
cosignature=d960fcff859a34d677343e4789c6843e897c9ff195ea7140a6ef382566df3b65 1770115653 c57cf9e049013712be39660bd01f98e4ec0736945e2100deaec96dc28dc5d47d080e7b1f2fff00140b10ec8f76727c984b97f77fab9502c09cd123fbaea3140e
cosignature=e4a6a1e4657d8d7a187cc0c20ed51055d88c72f340d29534939aee32d86b4021 1770115653 2118348af36b7f2e92a892bd4faa97f18d75b14cc5572ea162a1eb667263a46f95a04cc68e6bb0b4863f9d57ccf0dc4d9b63f9bf89dda9b549e515e107d6b70a
cosignature=1c997261f16e6e81d13f420900a2542a4b6a049c2d996324ee5d82a90ca3360c 1770115653 7c32cab8b6d7401705a67a387fc8f513cc0eff0e7083a8161b9a917eb9722e440a201ce868ddc144cfe097b0296520d1bcf047db89407b848e1ff1408a507009
cosignature=42351ad474b29c04187fd0c8c7670656386f323f02e9a4ef0a0055ec061ecac8 1770115653 e1345f7e486c5364246b90c48eebde71ddfca8d4d14b6179d5ee552ff607e034b3ced8277343eefbad0b1e1f245cfa90c57120296c1f997b883eb065d0369a00
cosignature=86b5414ae57f45c2953a074640bb5bedebad023925d4dc91a31de1350b710089 1770115653 6daf9f1a8168be8280239263271adbadb5b49af5cc83ebc724bdf21ce07b9d729b3f46c12f4340644e8a7f8810dd23cbb043569b11d91b12ad54047d54d6b706
cosignature=49c4cd6124b7c572f3354d854d50b2a4b057a750f786cf03103c09de339c4ea3 1770115653 c674fe660e91a737ef6c211880455db56123ce9427cf53f6ebcc44d69088169971ab72f58a7ff99192dee0f457ba8e1361f84aff6cd566292066a5771032c205

leaf_index=381377
node_hash=6c1da5f670e2775af9d9f4be5332b049503b325943cec56ba350350418f725fb
node_hash=e21e73ee8caf0d49cdcd2f334b3f64e85e4a151daf8d09b6034306bb21b88480
node_hash=f77bc4db00e509149b6e2fc0028d7107dd415929dc9972f32fe758ce39bcc9a0
node_hash=3acb38f01c633d917b899ed4e522a49a02bf20d358f98ca530e3a3065591e7f2
node_hash=889de80c543a5ae8e35430988dc120ac7edde74b776f9082f814ea88190a601f
node_hash=199f812b9f3667dec31f964098e32652477a2f3d458019b6f8f4acc645cf0131
node_hash=084580f8f6324d4ae42dbcb779502ab9fab77e0c2b92519fe089be72e38d60ed
node_hash=9ddbece4939d621df53f31e2729d5fa7802fd82f3edfb784483d8b7fa9cf41e2
node_hash=e1c7a90c09949c263807e5970aef47f9a06164b759995ab814aff94aff9dcd00"#;

    static RELAY_LIST_CONTENT: &str = r#"{"locations":{"ie-dub":{"country":"Ireland","city":"Dublin","latitude":53.35014,"longitude":-6.266155},"de-fra":{"country":"Germany","city":"Frankfurt","latitude":50.110924,"longitude":8.682127},"se-got":{"country":"Sweden","city":"Gothenburg","latitude":57.70887,"longitude":11.97456},"se-mma":{"country":"Sweden","city":"Malmö","latitude":55.607075,"longitude":13.002716},"aa-rsw":{"country":"Relay Software Country","city":"Relay Software city","latitude":0.0,"longitude":0.0},"se-sto":{"country":"Sweden","city":"Stockholm","latitude":59.3289,"longitude":18.0649}},"openvpn":{"relays":[],"ports":[{"port":1194,"protocol":"udp"},{"port":1195,"protocol":"udp"},{"port":1196,"protocol":"udp"},{"port":1197,"protocol":"udp"},{"port":1300,"protocol":"udp"},{"port":1301,"protocol":"udp"},{"port":1302,"protocol":"udp"},{"port":443,"protocol":"tcp"},{"port":80,"protocol":"tcp"}]},"wireguard":{"relays":[{"hostname":"de-fra-wg-001","location":"de-fra","active":true,"owned":false,"provider":"xtom","stboot":true,"ipv4_addr_in":"85.203.53.104","include_in_country":true,"weight":100,"public_key":"9NuXfdBjkHkVy5IdQN+8wMNS7CiFC3n+VRdFsPzgmVM=","ipv6_addr_in":"2a03:1b20:5:3::104","daita":true,"features":{"daita":{},"lwo":{},"quic":{"addr_in":["85.203.53.108","2a03:1b20:5:3::108"],"token":"28234bf5-c4ec-4f28-8975-7d7dc5d537c9","domain":"de-fra-wg-001.relays.stagemole.eu"}}},{"hostname":"ie-dub-wg-001","location":"ie-dub","active":true,"owned":false,"provider":"M247","stboot":true,"ipv4_addr_in":"85.203.53.102","include_in_country":true,"weight":100,"public_key":"PeXs6GHjC4yKfo+1giEN6gGDkae7wTo8hdZFT1kV3Ho=","ipv6_addr_in":"2a03:1b20:5:3::102","features":{"lwo":{},"quic":{"addr_in":["85.203.53.102","2a03:1b20:5:3::102"],"token":"28234bf5-c4ec-4f28-8975-7d7dc5d537c9","domain":"ie-dub-wg-001.relays.stagemole.eu"}}},{"hostname":"se-got-wg-001","location":"se-got","active":true,"owned":true,"provider":"31173","stboot":true,"ipv4_addr_in":"85.203.53.140","include_in_country":true,"weight":100,"public_key":"ZCvlPfPzOf728BIQcSmWzGFuInKK0SdVTyTCZkdrvUk=","ipv6_addr_in":"2a03:1b20:5:3::140","features":{"lwo":{},"quic":{"addr_in":["2a03:1b20:5:3::14ff"],"token":"28234bf5-c4ec-4f28-8975-7d7dc5d537c9","domain":"se-got-wg-001.relays.stagemole.eu"}}},{"hostname":"se-got-wg-002","location":"aa-rsw","active":true,"owned":true,"provider":"RelaySoftwareTeam","stboot":true,"ipv4_addr_in":"85.203.53.145","include_in_country":true,"weight":0,"public_key":"IyER1oEmmuiijmyjI2D4ihrDuButvK4B00h5Z3+0nRM=","ipv6_addr_in":"2a03:1b20:5:3::145","daita":true,"features":{"daita":{},"lwo":{}}},{"hostname":"se-got-wg-003","location":"aa-rsw","active":true,"owned":true,"provider":"RelaySoftwareTeam","stboot":true,"ipv4_addr_in":"85.203.53.147","include_in_country":true,"weight":0,"public_key":"f6A7xEIcAYhpxNgf2KPj76zlaU/ebqYewmmoIHL+ABQ=","ipv6_addr_in":"2a03:1b20:5:3::147","daita":true,"features":{"daita":{},"lwo":{}}},{"hostname":"se-got-wg-004","location":"aa-rsw","active":true,"owned":true,"provider":"RelaySoftwareTeam","stboot":true,"ipv4_addr_in":"85.203.53.231","include_in_country":true,"weight":0,"public_key":"VYtjgSxNzWi9uRaMlMilxjWeuBVQqdTguamP+Fcjj2o=","ipv6_addr_in":"2a03:1b20:5:3::231","daita":true,"features":{"daita":{}}},{"hostname":"se-got-wg-005","location":"se-got","active":false,"owned":true,"provider":"31173","stboot":true,"ipv4_addr_in":"85.203.53.222","include_in_country":true,"weight":100,"public_key":"czbIog4ERYNb8MkMAZmHZ6dC4Eg7tOAjqgJUgxd9Nnk=","ipv6_addr_in":"2a03:1b20:5:3::222","features":{"lwo":{}}},{"hostname":"se-sto-wg-001","location":"se-sto","active":true,"owned":true,"provider":"Mullvad","stboot":true,"ipv4_addr_in":"85.203.53.81","include_in_country":true,"weight":100,"public_key":"2KS+F8ZAOUSMwygl2CYqkqFhbi3L5u58b3kIpaylaEk=","ipv6_addr_in":"2a03:1b20:5:3::81","features":{"lwo":{},"quic":{"addr_in":["85.203.53.81","2a03:1b20:5:3::81"],"token":"28234bf5-c4ec-4f28-8975-7d7dc5d537c9","domain":"se-sto-wg-001.relays.stagemole.eu"}}}],"port_ranges":[[53,53],[123,123],[4000,33433],[33565,51820],[52001,60000]],"shadowsocks_port_ranges":[[51900,51949]],"ipv4_gateway":"10.64.0.1","ipv6_gateway":"fc00:bbbb:bbbb:bb01::1"},"bridge":{"relays":[{"hostname":"se-got-br-001","location":"se-got","active":true,"owned":true,"provider":"Mullvad","stboot":true,"ipv4_addr_in":"85.203.53.200","include_in_country":true,"weight":100}],"shadowsocks":[{"protocol":"tcp","port":443,"cipher":"aes-256-gcm","password":"mullvad"},{"protocol":"udp","port":1234,"cipher":"aes-256-cfb","password":"mullvad"},{"protocol":"udp","port":1236,"cipher":"aes-256-gcm","password":"mullvad"}]}}"#;
}
