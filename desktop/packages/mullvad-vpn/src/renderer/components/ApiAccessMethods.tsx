import { useCallback, useMemo } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { AccessMethodSetting } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import { useApiAccessMethodTest } from '../lib/api-access-methods';
import { Container, Flex, Spinner } from '../lib/components';
import { Colors, Spacings } from '../lib/foundations';
import { useHistory } from '../lib/history';
import { generateRoutePath } from '../lib/routeHelpers';
import { RoutePath } from '../lib/routes';
import { useBoolean } from '../lib/utility-hooks';
import { useSelector } from '../redux/store';
import { AppNavigationHeader } from './';
import * as Cell from './cell';
import {
  ContextMenu,
  ContextMenuContainer,
  ContextMenuItem,
  ContextMenuTrigger,
} from './ContextMenu';
import InfoButton from './InfoButton';
import { BackAction } from './KeyboardNavigation';
import { Layout, SettingsContainer, SettingsContent, SettingsNavigationScrollbars } from './Layout';
import { ModalAlert, ModalAlertType } from './Modal';
import { NavigationContainer } from './NavigationContainer';
import SettingsHeader, { HeaderSubTitle, HeaderTitle } from './SettingsHeader';
import { SmallButton, SmallButtonColor } from './SmallButton';

const StyledNameLabel = styled(Cell.Label)({
  display: 'block',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
});

const StyledTestResultCircle = styled.div<{ $result: boolean }>((props) => ({
  width: '10px',
  height: '10px',
  borderRadius: '50%',
  backgroundColor: props.$result ? Colors.green : Colors.red,
  marginRight: Spacings.spacing2,
}));

// This component is the topmost component in the API access methods view.
export default function ApiAccessMethods() {
  const history = useHistory();
  const methods = useSelector((state) => state.settings.apiAccessMethods);
  const currentMethod = useSelector((state) => state.settings.currentApiAccessMethod);

  const navigateToEdit = useCallback(
    (id?: string) => {
      const path = generateRoutePath(RoutePath.editApiAccessMethods, { id });
      history.push(path);
    },
    [history],
  );

  const navigateToNew = useCallback(() => navigateToEdit(), [navigateToEdit]);

  return (
    <BackAction action={history.pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader
              title={
                // TRANSLATORS: Title label in navigation bar
                messages.pgettext('navigation-bar', 'API access')
              }>
              <AppNavigationHeader.InfoButton
                message={[
                  messages.pgettext(
                    'api-access-methods-view',
                    'The app needs to communicate with a Mullvad API server to log you in, fetch server lists, and other critical operations.',
                  ),
                  messages.pgettext(
                    'api-access-methods-view',
                    'On some networks, where various types of censorship are being used, the API servers might not be directly reachable.',
                  ),
                  messages.pgettext(
                    'api-access-methods-view',
                    'This feature allows you to circumvent that censorship by adding custom ways to access the API via proxies and similar methods.',
                  ),
                ]}
              />
            </AppNavigationHeader>

            <SettingsNavigationScrollbars fillContainer>
              <SettingsContent>
                <SettingsHeader>
                  <HeaderTitle>{messages.pgettext('navigation-bar', 'API access')}</HeaderTitle>
                  <HeaderSubTitle>
                    {messages.pgettext(
                      'api-access-methods-view',
                      'Manage and add custom methods to access the Mullvad API.',
                    )}
                  </HeaderSubTitle>
                </SettingsHeader>

                <Flex $flexDirection="column" $gap={Spacings.spacing6}>
                  <Cell.Group $noMarginBottom>
                    <ApiAccessMethod
                      method={methods.direct}
                      inUse={methods.direct.id === currentMethod?.id}
                    />
                    <ApiAccessMethod
                      method={methods.mullvadBridges}
                      inUse={methods.mullvadBridges.id === currentMethod?.id}
                    />
                    <ApiAccessMethod
                      method={methods.encryptedDnsProxy}
                      inUse={methods.encryptedDnsProxy.id === currentMethod?.id}
                    />
                    {methods.custom.map((method) => (
                      <ApiAccessMethod
                        key={method.id}
                        method={method}
                        inUse={method.id === currentMethod?.id}
                        custom
                      />
                    ))}
                  </Cell.Group>
                  <Container size="4" $flex={1} $justifyContent="flex-end">
                    <SmallButton onClick={navigateToNew}>{messages.gettext('Add')}</SmallButton>
                  </Container>
                </Flex>
              </SettingsContent>
            </SettingsNavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

interface ApiAccessMethodProps {
  method: AccessMethodSetting;
  inUse: boolean;
  custom?: boolean;
}

function ApiAccessMethod(props: ApiAccessMethodProps) {
  const {
    setApiAccessMethod: setApiAccessMethodImpl,
    updateApiAccessMethod,
    removeApiAccessMethod,
  } = useAppContext();
  const { push } = useHistory();

  const [testing, testResult, testApiAccessMethod] = useApiAccessMethodTest();

  // State for delete confirmation dialog.
  const [removeConfirmationVisible, showRemoveConfirmation, hideRemoveConfirmation] = useBoolean();
  const confirmRemove = useCallback(() => {
    void removeApiAccessMethod(props.method.id);
    hideRemoveConfirmation();
  }, [hideRemoveConfirmation, props.method.id, removeApiAccessMethod]);

  // Toggle on/off on an access method.
  const toggle = useCallback(
    async (value: boolean) => {
      const updatedMethod = cloneMethod(props.method);
      updatedMethod.enabled = value;
      await updateApiAccessMethod(updatedMethod);
    },
    [props.method, updateApiAccessMethod],
  );

  const setApiAccessMethod = useCallback(async () => {
    const reachable = await testApiAccessMethod(props.method.id);
    if (reachable) {
      await setApiAccessMethodImpl(props.method.id);
    }
  }, [testApiAccessMethod, props.method.id, setApiAccessMethodImpl]);

  const menuItems = useMemo<Array<ContextMenuItem>>(() => {
    const items: Array<ContextMenuItem> = [
      {
        type: 'item' as const,
        label: messages.gettext('Use'),
        disabled: props.inUse,
        onClick: setApiAccessMethod,
      },
      {
        type: 'item' as const,
        label: messages.gettext('Test'),
        onClick: () => testApiAccessMethod(props.method.id),
      },
    ];

    // Edit and Delete shouldn't be available for direct, bridges or encrypted DNS proxy.
    if (props.custom) {
      items.push(
        { type: 'separator' as const },
        {
          type: 'item' as const,
          label: messages.gettext('Edit'),
          onClick: () =>
            push(generateRoutePath(RoutePath.editApiAccessMethods, { id: props.method.id })),
        },
        {
          type: 'item' as const,
          label: messages.gettext('Delete'),
          onClick: showRemoveConfirmation,
        },
      );
    }

    return items;
  }, [
    props.inUse,
    props.custom,
    props.method.id,
    setApiAccessMethod,
    testApiAccessMethod,
    showRemoveConfirmation,
    push,
  ]);

  return (
    <Cell.Row data-testid="access-method">
      <Cell.LabelContainer>
        <StyledNameLabel>{props.method.name}</StyledNameLabel>
        {testing && (
          <Cell.SubLabel>
            <Flex $gap={Spacings.spacing1} $alignItems="center">
              <Spinner size="small" />
              {messages.pgettext('api-access-methods-view', 'Testing...')}
            </Flex>
          </Cell.SubLabel>
        )}
        {!testing && testResult !== undefined && (
          <Cell.SubLabel>
            <StyledTestResultCircle $result={testResult} />
            {testResult
              ? messages.pgettext('api-access-methods-view', 'API reachable')
              : messages.pgettext('api-access-methods-view', 'API unreachable')}
          </Cell.SubLabel>
        )}
        {!testing && testResult === undefined && props.inUse && (
          <Cell.SubLabel>{messages.pgettext('api-access-methods-view', 'In use')}</Cell.SubLabel>
        )}
      </Cell.LabelContainer>
      <Flex $gap={Spacings.spacing3} $alignItems="center">
        {props.method.type === 'direct' && (
          <InfoButton
            message={[
              messages.pgettext(
                'api-access-methods-view',
                'With the “Direct” method, the app communicates with a Mullvad API server directly without any intermediate proxies.',
              ),
              messages.pgettext(
                'api-access-methods-view',
                'This can be useful when you are not affected by censorship.',
              ),
            ]}
          />
        )}
        {props.method.type === 'bridges' && (
          <InfoButton
            message={[
              messages.pgettext(
                'api-access-methods-view',
                'With the “Mullvad bridges” method, the app communicates with a Mullvad API server via a Mullvad bridge server. It does this by sending the traffic obfuscated by Shadowsocks.',
              ),
              messages.pgettext(
                'api-access-methods-view',
                'This can be useful if the API is censored but Mullvad’s bridge servers are not.',
              ),
            ]}
          />
        )}

        {props.method.type === 'encrypted-dns-proxy' && (
          <InfoButton
            message={[
              messages.pgettext(
                'api-access-methods-view',
                'With the “Encrypted DNS proxy” method, the app will communicate with our Mullvad API through a proxy address. It does this by retrieving an address from a DNS over HTTPS (DoH) server and then using that to reach our API servers.',
              ),
              messages.pgettext(
                'api-access-methods-view',
                'If you are not connected to our VPN, then the Encrypted DNS proxy will use your own non-VPN IP when connecting. The DoH servers are hosted by one of the following providers: Quad9 or CloudFlare.',
              ),
            ]}
          />
        )}
        <ContextMenuContainer>
          <ContextMenuTrigger />
          <ContextMenu items={menuItems} align="right" />
        </ContextMenuContainer>
        <Cell.Switch isOn={props.method.enabled} onChange={toggle} />
      </Flex>

      {/* Confirmation dialog for method removal */}
      <ModalAlert
        isOpen={removeConfirmationVisible}
        type={ModalAlertType.warning}
        gridButtons={[
          <SmallButton key="cancel" onClick={hideRemoveConfirmation}>
            {messages.gettext('Cancel')}
          </SmallButton>,
          <SmallButton key="confirm" onClick={confirmRemove} color={SmallButtonColor.red}>
            {messages.gettext('Delete')}
          </SmallButton>,
        ]}
        close={hideRemoveConfirmation}
        title={sprintf(messages.pgettext('api-access-methods-view', 'Delete %(name)s?'), {
          name: props.method.name,
        })}
        message={
          props.inUse
            ? messages.pgettext(
                'api-access-methods-view',
                'The in use API access method will change.',
              )
            : undefined
        }
      />
    </Cell.Row>
  );
}

function cloneMethod<T extends AccessMethodSetting>(method: T): T {
  const clonedMethod = {
    ...method,
  };

  if (
    method.type === 'socks5-remote' &&
    clonedMethod.type === 'socks5-remote' &&
    method.authentication !== undefined
  ) {
    clonedMethod.authentication = { ...method.authentication };
  }

  return clonedMethod;
}
