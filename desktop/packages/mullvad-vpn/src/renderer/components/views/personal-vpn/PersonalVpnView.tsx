import React, { useCallback, useEffect, useMemo, useState } from 'react';
import styled from 'styled-components';

import { PersonalVpnConfig } from '../../../../shared/daemon-rpc-types';
import { messages } from '../../../../shared/gettext';
import log from '../../../../shared/logging';
import { usePersonalVpn } from '../../../features/personal-vpn/hooks';
import {
  EndpointValidationError,
  KeyValidationError,
  validateAllowedIp,
  validateEndpoint,
  validateIp,
  validateWireguardKey,
} from '../../../features/personal-vpn/lib/validate';
import { Button, Flex, Text } from '../../../lib/components';
import { FlexColumn } from '../../../lib/components/flex-column';
import { Switch } from '../../../lib/components/switch';
import { View } from '../../../lib/components/view';
import { colors, spacings } from '../../../lib/foundations';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { HeaderTitle } from '../../SettingsHeader';

type FormState = {
  privateKey: string;
  tunnelIps: string[];
  publicKey: string;
  allowedIps: string[];
  endpoint: string;
};

type FormErrors = {
  privateKey?: KeyValidationError;
  tunnelIps: Record<number, 'empty' | 'invalid'>;
  publicKey?: KeyValidationError;
  allowedIps: Record<number, 'empty'>;
  endpoint?: EndpointValidationError;
  submit?: string;
};

const EMPTY_ERRORS: FormErrors = { tunnelIps: {}, allowedIps: {} };

function fromConfig(config?: PersonalVpnConfig): FormState {
  return {
    privateKey: config?.tunnel?.privateKey ?? '',
    tunnelIps:
      config?.tunnel?.tunnelIps && config.tunnel.tunnelIps.length > 0
        ? [...config.tunnel.tunnelIps]
        : [''],
    publicKey: config?.peer?.publicKey ?? '',
    allowedIps:
      config?.peer?.allowedIps && config.peer.allowedIps.length > 0
        ? [...config.peer.allowedIps]
        : [''],
    endpoint: config?.peer?.endpoint ?? '',
  };
}

const StyledSection = styled.div({
  display: 'flex',
  flexDirection: 'column',
  gap: spacings.tiny,
  background: colors.blue40,
  borderRadius: '8px',
  padding: spacings.medium,
});

const StyledSectionTitle = styled(Text)({
  marginBottom: spacings.small,
});

const StyledFieldLabel = styled(Text)({
  marginTop: spacings.small,
});

const StyledInput = styled.input<{ $invalid?: boolean }>(({ $invalid }) => ({
  background: colors.blue20,
  border: `1px solid ${$invalid ? colors.red : colors.transparent}`,
  borderRadius: '4px',
  color: colors.white,
  padding: `${spacings.small} ${spacings.medium}`,
  fontSize: '14px',
  width: '100%',
  outline: 'none',
  '&:focus': {
    borderColor: $invalid ? colors.red : colors.blue,
  },
  '&::placeholder': {
    color: colors.whiteAlpha60,
  },
  '&:disabled': {
    opacity: 0.5,
  },
}));

const StyledErrorText = styled(Text).attrs({ variant: 'labelTiny', color: 'red' })({
  marginTop: spacings.tiny,
});

const StyledStatsRow = styled(Flex)({
  justifyContent: 'space-between',
});

function keyErrorMessage(error: KeyValidationError): string {
  return error === 'empty'
    ? messages.pgettext('personal-vpn-view', 'Required')
    : messages.pgettext('personal-vpn-view', 'Expected a 32-byte base64 key');
}

function ipErrorMessage(error: 'empty' | 'invalid'): string {
  return error === 'empty'
    ? messages.pgettext('personal-vpn-view', 'Required')
    : messages.pgettext('personal-vpn-view', 'Not a valid IP address');
}

function endpointErrorMessage(error: EndpointValidationError): string {
  switch (error) {
    case 'empty':
      return messages.pgettext('personal-vpn-view', 'Required');
    case 'invalid-address':
      return messages.pgettext('personal-vpn-view', 'Expected <ip>:<port>');
    case 'invalid-port':
      return messages.pgettext('personal-vpn-view', 'Port must be between 1 and 65535');
  }
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KiB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MiB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GiB`;
}

function formatHandshake(timestamp?: string): string {
  if (!timestamp) return messages.pgettext('personal-vpn-view', 'Never');
  const ms = Date.now() - new Date(timestamp).getTime();
  if (ms < 0) return messages.pgettext('personal-vpn-view', 'Just now');
  const seconds = Math.floor(ms / 1000);
  if (seconds < 60) return `${seconds}s ago`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  return `${hours}h ago`;
}

interface IpRowProps {
  index: number;
  value: string;
  invalid: boolean;
  placeholder: string;
  onChange: (index: number, value: string) => void;
  onRemove: (index: number) => void;
  removeDisabled: boolean;
}

const IpRow = React.memo(function IpRow(props: IpRowProps) {
  const { index, value, invalid, placeholder, onChange, onRemove, removeDisabled } = props;

  const handleChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => onChange(index, event.target.value),
    [index, onChange],
  );
  const handleRemove = useCallback(() => onRemove(index), [index, onRemove]);

  return (
    <Flex gap="small">
      <StyledInput
        spellCheck={false}
        placeholder={placeholder}
        value={value}
        $invalid={invalid}
        onChange={handleChange}
      />
      <Button variant="primary" onClick={handleRemove} disabled={removeDisabled}>
        <Button.Text>−</Button.Text>
      </Button>
    </Flex>
  );
});

export function PersonalVpnView() {
  const { pop } = useHistory();
  const { config, enabled, stats, save, setEnabled, clear } = usePersonalVpn();

  const [form, setForm] = useState<FormState>(() => fromConfig(config));
  const [errors, setErrors] = useState<FormErrors>(EMPTY_ERRORS);
  const [saving, setSaving] = useState(false);

  // Reset the form when the persisted config changes (e.g. first load or an external edit).
  // `config` is a memoised selector result, so the reference only changes when the
  // underlying settings change.
  useEffect(() => {
    setForm(fromConfig(config));
    setErrors(EMPTY_ERRORS);
  }, [config]);

  const clearFieldError = useCallback((key: keyof FormErrors) => {
    setErrors((prev) => ({ ...prev, [key]: undefined, submit: undefined }));
  }, []);

  const onPrivateKeyChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const value = event.target.value;
      setForm((prev) => ({ ...prev, privateKey: value }));
      clearFieldError('privateKey');
    },
    [clearFieldError],
  );

  const updateTunnelIp = useCallback((index: number, value: string) => {
    setForm((prev) => {
      const next = [...prev.tunnelIps];
      next[index] = value;
      return { ...prev, tunnelIps: next };
    });
    setErrors((prev) => {
      const next = { ...prev.tunnelIps };
      delete next[index];
      return { ...prev, tunnelIps: next, submit: undefined };
    });
  }, []);

  const addTunnelIp = useCallback(() => {
    setForm((prev) => ({ ...prev, tunnelIps: [...prev.tunnelIps, ''] }));
  }, []);

  const removeTunnelIp = useCallback((index: number) => {
    setForm((prev) => {
      const next = prev.tunnelIps.filter((_, i) => i !== index);
      return { ...prev, tunnelIps: next.length === 0 ? [''] : next };
    });
    setErrors((prev) => {
      const next: Record<number, 'empty' | 'invalid'> = {};
      for (const [k, v] of Object.entries(prev.tunnelIps)) {
        const i = Number(k);
        if (i < index) next[i] = v;
        else if (i > index) next[i - 1] = v;
      }
      return { ...prev, tunnelIps: next };
    });
  }, []);

  const onPublicKeyChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const value = event.target.value;
      setForm((prev) => ({ ...prev, publicKey: value }));
      clearFieldError('publicKey');
    },
    [clearFieldError],
  );

  const onEndpointChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const value = event.target.value;
      setForm((prev) => ({ ...prev, endpoint: value }));
      clearFieldError('endpoint');
    },
    [clearFieldError],
  );

  const updateAllowedIp = useCallback((index: number, value: string) => {
    setForm((prev) => {
      const next = [...prev.allowedIps];
      next[index] = value;
      return { ...prev, allowedIps: next };
    });
    setErrors((prev) => {
      const next = { ...prev.allowedIps };
      delete next[index];
      return { ...prev, allowedIps: next, submit: undefined };
    });
  }, []);

  const addAllowedIp = useCallback(() => {
    setForm((prev) => ({ ...prev, allowedIps: [...prev.allowedIps, ''] }));
  }, []);

  const removeAllowedIp = useCallback((index: number) => {
    setForm((prev) => {
      const next = prev.allowedIps.filter((_, i) => i !== index);
      return { ...prev, allowedIps: next.length === 0 ? [''] : next };
    });
    setErrors((prev) => {
      const next: Record<number, 'empty'> = {};
      for (const [k, v] of Object.entries(prev.allowedIps)) {
        const i = Number(k);
        if (i < index) next[i] = v;
        else if (i > index) next[i - 1] = v;
      }
      return { ...prev, allowedIps: next };
    });
  }, []);

  const onSave = useCallback(async () => {
    const next: FormErrors = { tunnelIps: {}, allowedIps: {} };
    next.privateKey = validateWireguardKey(form.privateKey);
    next.publicKey = validateWireguardKey(form.publicKey);
    next.endpoint = validateEndpoint(form.endpoint);
    form.tunnelIps.forEach((ip, i) => {
      const err = validateIp(ip);
      if (err) next.tunnelIps[i] = err;
    });
    form.allowedIps.forEach((ip, i) => {
      const err = validateAllowedIp(ip);
      if (err) next.allowedIps[i] = err;
    });

    const hasError =
      next.privateKey !== undefined ||
      next.publicKey !== undefined ||
      next.endpoint !== undefined ||
      Object.keys(next.tunnelIps).length > 0 ||
      Object.keys(next.allowedIps).length > 0;

    if (hasError) {
      setErrors(next);
      return;
    }

    const payload: PersonalVpnConfig = {
      tunnel: {
        privateKey: form.privateKey,
        tunnelIps: form.tunnelIps.map((ip) => ip.trim()),
      },
      peer: {
        publicKey: form.publicKey,
        allowedIps: form.allowedIps.map((ip) => ip.trim()),
        endpoint: form.endpoint.trim(),
      },
    };

    setSaving(true);
    try {
      const result = await save(payload);
      if (result.type === 'error') {
        setErrors({ tunnelIps: {}, allowedIps: {}, submit: result.message });
      } else {
        setErrors(EMPTY_ERRORS);
      }
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to save personal VPN config: ${error.message}`);
      setErrors({ tunnelIps: {}, allowedIps: {}, submit: error.message });
    } finally {
      setSaving(false);
    }
  }, [form, save]);

  const onClear = useCallback(async () => {
    try {
      await clear();
      setForm(fromConfig(undefined));
      setErrors(EMPTY_ERRORS);
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to clear personal VPN config: ${error.message}`);
    }
  }, [clear]);

  const canToggle = config?.tunnel !== undefined && config?.peer !== undefined;
  const allowedIpsLength = form.allowedIps.length;
  const tunnelIpsLength = form.tunnelIps.length;

  const isDirty = useMemo(() => {
    const saved = fromConfig(config);
    return (
      saved.privateKey !== form.privateKey ||
      saved.publicKey !== form.publicKey ||
      saved.endpoint !== form.endpoint ||
      saved.tunnelIps.length !== form.tunnelIps.length ||
      saved.tunnelIps.some((ip, i) => ip !== form.tunnelIps[i]) ||
      saved.allowedIps.length !== form.allowedIps.length ||
      saved.allowedIps.some((ip, i) => ip !== form.allowedIps[i])
    );
  }, [config, form]);

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={pop}>
        <NavigationContainer>
          <AppNavigationHeader title={messages.pgettext('personal-vpn-view', 'Personal VPN')} />

          <NavigationScrollbars>
            <View.Content>
              <View.Container horizontalMargin="medium" flexDirection="column" gap="medium">
                <HeaderTitle>{messages.pgettext('personal-vpn-view', 'Personal VPN')}</HeaderTitle>
                <Text variant="labelTiny" color="whiteAlpha60">
                  {messages.pgettext(
                    'personal-vpn-view',
                    'Manually configure a WireGuard tunnel to a server you control, via the selected Mullvad relay.',
                  )}
                </Text>

                <StyledSection>
                  <Flex justifyContent="space-between" alignItems="center">
                    <Text variant="titleMedium">
                      {messages.pgettext('personal-vpn-view', 'Enable')}
                    </Text>
                    <Switch checked={enabled} onCheckedChange={setEnabled} disabled={!canToggle}>
                      <Switch.Trigger>
                        <Switch.Input />
                      </Switch.Trigger>
                    </Switch>
                  </Flex>
                  {!canToggle && (
                    <Text variant="labelTiny" color="whiteAlpha60">
                      {messages.pgettext(
                        'personal-vpn-view',
                        'Save a valid configuration before enabling.',
                      )}
                    </Text>
                  )}
                </StyledSection>

                <StyledSection>
                  <StyledSectionTitle variant="titleMedium">
                    {messages.pgettext('personal-vpn-view', 'Interface')}
                  </StyledSectionTitle>

                  <StyledFieldLabel variant="labelTiny" color="whiteAlpha60">
                    {messages.pgettext('personal-vpn-view', 'Private key')}
                  </StyledFieldLabel>
                  <StyledInput
                    type="password"
                    autoComplete="off"
                    spellCheck={false}
                    placeholder="base64"
                    value={form.privateKey}
                    $invalid={errors.privateKey !== undefined}
                    onChange={onPrivateKeyChange}
                  />
                  {errors.privateKey && (
                    <StyledErrorText>{keyErrorMessage(errors.privateKey)}</StyledErrorText>
                  )}

                  <StyledFieldLabel variant="labelTiny" color="whiteAlpha60">
                    {messages.pgettext('personal-vpn-view', 'Addresses')}
                  </StyledFieldLabel>
                  <FlexColumn gap="small">
                    {form.tunnelIps.map((ip, index) => (
                      <IpRow
                        key={index}
                        index={index}
                        value={ip}
                        placeholder="10.0.0.2"
                        invalid={errors.tunnelIps[index] !== undefined}
                        onChange={updateTunnelIp}
                        onRemove={removeTunnelIp}
                        removeDisabled={tunnelIpsLength === 1 && ip === ''}
                      />
                    ))}
                    <Button variant="primary" onClick={addTunnelIp}>
                      <Button.Text>
                        {messages.pgettext('personal-vpn-view', 'Add address')}
                      </Button.Text>
                    </Button>
                  </FlexColumn>
                  {Object.values(errors.tunnelIps).some((e) => e !== undefined) && (
                    <StyledErrorText>
                      {ipErrorMessage(
                        Object.values(errors.tunnelIps).find((e) => e !== undefined)!,
                      )}
                    </StyledErrorText>
                  )}
                </StyledSection>

                <StyledSection>
                  <StyledSectionTitle variant="titleMedium">
                    {messages.pgettext('personal-vpn-view', 'Peer')}
                  </StyledSectionTitle>

                  <StyledFieldLabel variant="labelTiny" color="whiteAlpha60">
                    {messages.pgettext('personal-vpn-view', 'Public key')}
                  </StyledFieldLabel>
                  <StyledInput
                    spellCheck={false}
                    placeholder="base64"
                    value={form.publicKey}
                    $invalid={errors.publicKey !== undefined}
                    onChange={onPublicKeyChange}
                  />
                  {errors.publicKey && (
                    <StyledErrorText>{keyErrorMessage(errors.publicKey)}</StyledErrorText>
                  )}

                  <StyledFieldLabel variant="labelTiny" color="whiteAlpha60">
                    {messages.pgettext('personal-vpn-view', 'Allowed IPs')}
                  </StyledFieldLabel>
                  <FlexColumn gap="small">
                    {form.allowedIps.map((ip, index) => (
                      <IpRow
                        key={index}
                        index={index}
                        value={ip}
                        placeholder="0.0.0.0/0"
                        invalid={errors.allowedIps[index] !== undefined}
                        onChange={updateAllowedIp}
                        onRemove={removeAllowedIp}
                        removeDisabled={allowedIpsLength === 1 && ip === ''}
                      />
                    ))}
                    <Button variant="primary" onClick={addAllowedIp}>
                      <Button.Text>
                        {messages.pgettext('personal-vpn-view', 'Add allowed IP')}
                      </Button.Text>
                    </Button>
                  </FlexColumn>

                  <StyledFieldLabel variant="labelTiny" color="whiteAlpha60">
                    {messages.pgettext('personal-vpn-view', 'Endpoint')}
                  </StyledFieldLabel>
                  <StyledInput
                    spellCheck={false}
                    placeholder="1.2.3.4:51820"
                    value={form.endpoint}
                    $invalid={errors.endpoint !== undefined}
                    onChange={onEndpointChange}
                  />
                  {errors.endpoint && (
                    <StyledErrorText>{endpointErrorMessage(errors.endpoint)}</StyledErrorText>
                  )}
                </StyledSection>

                {enabled && stats && (
                  <StyledSection>
                    <StyledSectionTitle variant="titleMedium">
                      {messages.pgettext('personal-vpn-view', 'Statistics')}
                    </StyledSectionTitle>
                    <StyledStatsRow>
                      <Text variant="labelTiny" color="whiteAlpha60">
                        {messages.pgettext('personal-vpn-view', 'Received')}
                      </Text>
                      <Text variant="labelTiny">{formatBytes(stats.rxBytes)}</Text>
                    </StyledStatsRow>
                    <StyledStatsRow>
                      <Text variant="labelTiny" color="whiteAlpha60">
                        {messages.pgettext('personal-vpn-view', 'Sent')}
                      </Text>
                      <Text variant="labelTiny">{formatBytes(stats.txBytes)}</Text>
                    </StyledStatsRow>
                    <StyledStatsRow>
                      <Text variant="labelTiny" color="whiteAlpha60">
                        {messages.pgettext('personal-vpn-view', 'Last handshake')}
                      </Text>
                      <Text variant="labelTiny">{formatHandshake(stats.lastHandshakeTime)}</Text>
                    </StyledStatsRow>
                  </StyledSection>
                )}

                {errors.submit && <StyledErrorText>{errors.submit}</StyledErrorText>}

                <FlexColumn gap="small">
                  <Button variant="success" onClick={onSave} disabled={saving || !isDirty}>
                    <Button.Text>{messages.pgettext('personal-vpn-view', 'Save')}</Button.Text>
                  </Button>
                  <Button variant="destructive" onClick={onClear} disabled={saving}>
                    <Button.Text>{messages.pgettext('personal-vpn-view', 'Clear')}</Button.Text>
                  </Button>
                </FlexColumn>
              </View.Container>
            </View.Content>
          </NavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}
