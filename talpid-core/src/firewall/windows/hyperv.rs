use wmi::{IWbemClassWrapper, WMIConnection, WMIError};

/// Name of the blocking Hyper-V rule.
const BLOCK_OUTBOUND_RULE_ELEMENT_NAME: &str = "Mullvad VPN outbound block-all rule";

/// Name of the blocking Hyper-V rule.
const BLOCK_INBOUND_RULE_ELEMENT_NAME: &str = "Mullvad VPN inbound block-all rule";

/// Unique instance ID identifying the outbound blocking Hyper-V rule.
const BLOCK_OUTBOUND_RULE_UUID: &str = "{319400cb-0445-4c1b-a081-1cbc57cdbcb8}";

/// Unique instance ID identifying the inbound blocking Hyper-V rule.
const BLOCK_INBOUND_RULE_UUID: &str = "{95a5e2c6-ebd5-45e5-9495-12c5d807cd91}";

const WMI_NAMESPACE: &str = "root\\standardcimv2";

/// HRESULT returned when a WMI object is not found.
/// See <https://learn.microsoft.com/en-us/windows/win32/wmisdk/wmi-error-constants>.
const WBEM_E_NOT_FOUND: i32 = 0x80041002u32 as i32;

/// Errors occurring while configuring Hyper-V firewall rules
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to connect to the WMI namespace '{WMI_NAMESPACE}'")]
    ConnectWmi(#[source] WMIError),
    #[error("Failed to obtain Hyper-V rule class")]
    ObtainHyperVClass(#[source] WMIError),
    #[error("Failed to create new instance of Hyper-V rule class")]
    NewRuleInstance(#[source] WMIError),
    #[error("Failed to set rule setting: {0}")]
    SetRuleKey(&'static str, #[source] WMIError),
    #[error(r#"Failed to put the rule "{0}""#)]
    PutInstance(&'static str, #[source] WMIError),
    #[error(r#"Failed to delete rule "{0}""#)]
    DeleteInstance(&'static str, #[source] WMIError),
}

/// Initialize WMI connection to the ROOT\StandardCIMV2 namespace, which may be used for
/// interacting with Hyper-V rules.
pub fn init_wmi() -> Result<WMIConnection, Error> {
    let con = WMIConnection::with_namespace_path(WMI_NAMESPACE).map_err(Error::ConnectWmi)?;

    // Test whether the class is available
    let _ = con
        .get_object("MSFT_NetFirewallHyperVRule")
        .map_err(Error::ObtainHyperVClass)?;

    Ok(con)
}

/// Add a Hyper-V rule that blocks all traffic using WMI (Windows Management Instrumentation).
///
/// Instances of the WMI class `MSFT_NetFirewallHyperVRule` in the namespace "root\standardcimv2"
/// belong to the same firewall ruleset as that visible in PowerShell using the command
/// `Get-NetFirewallHyperVRule`.
///
/// Details about the `MSFT_NetFirewallHyperVRule`, including the meaning of properties, are
/// documented here:
/// <https://learn.microsoft.com/en-us/windows/win32/fwp/wmi/wfascimprov/msft-netfirewallhypervrule>
///
/// `con` must be a valid WMI connection for the `root\standardcimv2` WMI namespace. Such a connection
/// can be initialized using [`init_wmi`].
pub fn add_blocking_hyperv_firewall_rules(con: &WMIConnection) -> Result<(), Error> {
    let class = con
        .get_object("MSFT_NetFirewallHyperVRule")
        .map_err(Error::ObtainHyperVClass)?;

    add_blocking_rule(
        con,
        &class,
        BLOCK_OUTBOUND_RULE_ELEMENT_NAME,
        BLOCK_OUTBOUND_RULE_UUID,
        Direction::Outbound,
    )?;
    add_blocking_rule(
        con,
        &class,
        BLOCK_INBOUND_RULE_ELEMENT_NAME,
        BLOCK_INBOUND_RULE_UUID,
        Direction::Inbound,
    )
}

#[repr(i32)]
enum Direction {
    Inbound = 1,
    Outbound = 2,
}

fn add_blocking_rule(
    con: &WMIConnection,
    rule_class: &IWbemClassWrapper,
    element_name: &'static str,
    instance_id: &str,
    direction: Direction,
) -> Result<(), Error> {
    let instance = rule_class
        .spawn_instance()
        .map_err(Error::NewRuleInstance)?;

    instance
        .put_property("ElementName", element_name)
        .map_err(|err| Error::SetRuleKey("ElementName", err))?;
    instance
        .put_property("InstanceID", instance_id)
        .map_err(|err| Error::SetRuleKey("InstanceID", err))?;

    // Action: 4 = block
    instance
        .put_property("Action", 4i32)
        .map_err(|err| Error::SetRuleKey("Action", err))?;

    // Enabled: 1 = enabled
    instance
        .put_property("Enabled", 1i32)
        .map_err(|err| Error::SetRuleKey("Enabled", err))?;

    instance
        .put_property("Direction", direction as i32)
        .map_err(|err| Error::SetRuleKey("Direction", err))?;

    con.put_instance(&instance)
        .map_err(|error| Error::PutInstance(element_name, error))
}

/// Remove Hyper-V rule previously added by [`add_blocking_hyperv_firewall_rules`]. See the
/// documentation of that function for more details.
///
/// This function succeeds if the rule is not present or has already been removed.
///
/// `con` must be a valid WMI connection for the `root\standardcimv2` WMI namespace. Such a connection
/// can be initialized using [`init_wmi`].
pub fn remove_blocking_hyperv_firewall_rules(con: &WMIConnection) -> Result<(), Error> {
    remove_blocking_rule(
        con,
        BLOCK_INBOUND_RULE_ELEMENT_NAME,
        BLOCK_INBOUND_RULE_UUID,
    )?;
    remove_blocking_rule(
        con,
        BLOCK_OUTBOUND_RULE_ELEMENT_NAME,
        BLOCK_OUTBOUND_RULE_UUID,
    )
}

fn remove_blocking_rule(
    con: &WMIConnection,
    element_name: &'static str,
    instance_id: &str,
) -> Result<(), Error> {
    let rule_path = format!(r#"MSFT_NetFirewallHyperVRule.InstanceID="{instance_id}""#);
    con.delete_instance(&rule_path)
        .or_else(|error| map_deletion_err(element_name, error))
}

fn map_deletion_err(element_name: &'static str, err: WMIError) -> Result<(), Error> {
    if matches!(err, WMIError::HResultError { hres } if hres == WBEM_E_NOT_FOUND) {
        // If the rule doesn't exist, do nothing
        Ok(())
    } else {
        Err(Error::DeleteInstance(element_name, err))
    }
}
