import React, { useCallback, useState } from 'react';
import styled from 'styled-components';

import { strings } from '../../../../shared/constants';
import { messages } from '../../../../shared/gettext';
import { useAppContext } from '../../../context';
import { Flex, Icon, IconButton } from '../../../lib/components';
import { AnimatedList } from '../../../lib/components/animated-list';
import { View } from '../../../lib/components/view';
import { colors, spacings } from '../../../lib/foundations';
import { usePop } from '../../../history/hooks';
import { useSelector } from '../../../redux/store';
import { AppNavigationHeader } from '../../';
import * as Cell from '../../cell';
import { normalText } from '../../common-styles';
import { BackAction } from '../../keyboard-navigation';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { NavigationContainer } from '../../NavigationContainer';
import SettingsHeader, { HeaderSubTitle, HeaderTitle } from '../../SettingsHeader';

const StyledSearchContainer = styled.div({
  position: 'relative',
  display: 'flex',
  marginLeft: spacings.medium,
  marginRight: spacings.medium,
  marginBottom: spacings.medium,
});

const StyledSearchInput = styled.input.attrs({ type: 'text' })({
  ...normalText,
  flex: 1,
  border: 'none',
  borderRadius: '4px',
  padding: '9px 38px 9px 12px',
  margin: 0,
  lineHeight: '24px',
  color: colors.whiteAlpha60,
  backgroundColor: colors.whiteOnDarkBlue10,
  outline: 'none',
  '&&::placeholder': {
    color: colors.whiteOnDarkBlue60,
  },
  '&&:focus': {
    color: colors.whiteAlpha60,
    backgroundColor: colors.whiteOnDarkBlue10,
  },
});

const StyledAddIcon = styled(Icon)({
  position: 'absolute',
  top: '50%',
  transform: 'translateY(-50%)',
  right: '9px',
  backgroundColor: colors.whiteOnDarkBlue60,
  cursor: 'pointer',
  '&&:hover': {
    backgroundColor: colors.whiteOnDarkBlue40,
  },
});

const StyledRowContainer = styled(Cell.Container)({
  backgroundColor: colors.blue40,
});

export function SplitTunnelingIpView() {
  const pop = usePop();
  const ipExclusions = useSelector((state) => state.settings.splitTunnelingIpExclusions);
  const { addSplitTunnelIpNetwork, removeSplitTunnelIpNetwork } = useAppContext();
  const [inputValue, setInputValue] = useState('');
  const [removedIps, setRemovedIps] = useState<string[]>([]);

  if (window.env.platform !== 'win32') {
    return null;
  }

  const validateCidr = (value: string): boolean => {
    const ipv4Cidr = /^(\d{1,3}\.){3}\d{1,3}(\/\d{1,2})?$/;
    const ipv6Cidr = /^[0-9a-fA-F:.]+(\/\d{1,3})?$/;
    return ipv4Cidr.test(value) || ipv6Cidr.test(value);
  };

  const normalizeNetwork = (value: string): string => {
    if (!value.includes('/')) {
      return value.includes(':') ? `${value}/128` : `${value}/32`;
    }
    return value;
  };

  const isValidInput = (): boolean => {
    const trimmed = inputValue.trim();
    if (!trimmed) return false;
    if (!validateCidr(trimmed)) return false;
    const network = normalizeNetwork(trimmed);
    if (ipExclusions.includes(network)) return false;
    return true;
  };

  const onAdd = async () => {
    const trimmed = inputValue.trim();
    if (!trimmed) return;
    if (!validateCidr(trimmed)) return;

    const network = normalizeNetwork(trimmed);
    if (ipExclusions.includes(network)) return;

    try {
      await addSplitTunnelIpNetwork(network);
      setInputValue('');
      setRemovedIps((prev) => prev.filter((ip) => ip !== network));
    } catch {
      // silently fail
    }
  };

  const onRemoveFromIncluded = useCallback(
    (network: string) => {
      void removeSplitTunnelIpNetwork(network);
      setRemovedIps((prev) => (prev.includes(network) ? prev : [...prev, network]));
    },
    [removeSplitTunnelIpNetwork],
  );

  const onRestoreFromExcluded = useCallback(
    async (network: string) => {
      try {
        await addSplitTunnelIpNetwork(network);
        setRemovedIps((prev) => prev.filter((ip) => ip !== network));
      } catch {
        // silently fail
      }
    },
    [addSplitTunnelIpNetwork],
  );

  const onPermanentlyRemove = useCallback((network: string) => {
    setRemovedIps((prev) => prev.filter((ip) => ip !== network));
  }, []);

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      void onAdd();
    }
  };

  const onInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setInputValue(e.target.value);
  };

  const includedTitle = (
    <Cell.SectionTitle>
      {
        // TRANSLATORS: Section title for IP networks included in split tunneling.
        messages.pgettext('split-tunneling-ip-view', 'Included IPs')
      }
    </Cell.SectionTitle>
  );
  const excludedTitle = (
    <Cell.SectionTitle>
      {
        // TRANSLATORS: Section title for IP networks removed from split tunneling.
        messages.pgettext('split-tunneling-ip-view', 'Excluded IPs')
      }
    </Cell.SectionTitle>
  );

  const actualRemovedIps = removedIps.filter((ip) => !ipExclusions.includes(ip));
  const canAdd = isValidInput();

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={pop}>
        <NavigationContainer>
          <AppNavigationHeader
            title={
              // TRANSLATORS: Title label in navigation bar
              strings.splitTunnelingIp
            }
          />

          <NavigationScrollbars fillContainer>
            <View.Content>
              <SettingsHeader>
                <HeaderTitle>{strings.splitTunnelingIp}</HeaderTitle>
                <HeaderSubTitle>
                  {messages.pgettext(
                    'split-tunneling-ip-view',
                    'Exclude traffic to specific IP addresses or subnets from the VPN tunnel (e.g. 100.64.0.0/10).',
                  )}
                </HeaderSubTitle>
              </SettingsHeader>

              <StyledSearchContainer>
                <StyledSearchInput
                  value={inputValue}
                  onInput={onInputChange}
                  onKeyDown={onKeyDown}
                  placeholder={messages.pgettext(
                    'split-tunneling-ip-view',
                    'Add IP or subnet...',
                  )}
                />
                <StyledAddIcon
                  icon={canAdd ? 'add-circle' : 'alert-circle'}
                  onClick={() => canAdd && void onAdd()}
                />
              </StyledSearchContainer>

              <Flex flexDirection="column" gap="medium">
                <Cell.Section sectionTitle={includedTitle}>
                  <AnimatedList>
                    {ipExclusions.map((network) => (
                      <AnimatedList.Item key={network}>
                        <StyledRowContainer>
                          <Cell.Label>{network}</Cell.Label>
                          <IconButton
                            variant="secondary"
                            onClick={() => onRemoveFromIncluded(network)}
                            aria-label={`Remove ${network}`}>
                            <IconButton.Icon icon="remove-circle" />
                          </IconButton>
                        </StyledRowContainer>
                      </AnimatedList.Item>
                    ))}
                  </AnimatedList>
                </Cell.Section>

                <Cell.Section sectionTitle={excludedTitle}>
                  <AnimatedList>
                    {actualRemovedIps.map((network) => (
                      <AnimatedList.Item key={network}>
                        <StyledRowContainer>
                          <Cell.Label>{network}</Cell.Label>
                          <Flex gap="small">
                            <IconButton
                              variant="secondary"
                              onClick={() => void onRestoreFromExcluded(network)}
                              aria-label={`Restore ${network}`}>
                              <IconButton.Icon icon="add-circle" />
                            </IconButton>
                            <IconButton
                              variant="secondary"
                              onClick={() => onPermanentlyRemove(network)}
                              aria-label={`Permanently remove ${network}`}>
                              <IconButton.Icon icon="remove-circle" />
                            </IconButton>
                          </Flex>
                        </StyledRowContainer>
                      </AnimatedList.Item>
                    ))}
                  </AnimatedList>
                </Cell.Section>
              </Flex>
            </View.Content>
          </NavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}
