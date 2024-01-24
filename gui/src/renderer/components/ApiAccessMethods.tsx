import { useCallback, useMemo } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { colors } from '../../config.json';
import { AccessMethodSetting } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import { useApiAccessMethodTest } from '../lib/api-access-methods';
import { useHistory } from '../lib/history';
import { generateRoutePath } from '../lib/routeHelpers';
import { RoutePath } from '../lib/routes';
import { useBoolean } from '../lib/utilityHooks';
import { useSelector } from '../redux/store';
import * as Cell from './cell';
import {
  ContextMenu,
  ContextMenuContainer,
  ContextMenuItem,
  ContextMenuTrigger,
} from './ContextMenu';
import ImageView from './ImageView';
import InfoButton from './InfoButton';
import { BackAction } from './KeyboardNavigation';
import { Layout, SettingsContainer } from './Layout';
import { ModalAlert, ModalAlertType } from './Modal';
import { NavigationBar, NavigationContainer, NavigationItems, TitleBarItem } from './NavigationBar';
import SettingsHeader, { HeaderSubTitle, HeaderTitle } from './SettingsHeader';
import { StyledContent, StyledNavigationScrollbars, StyledSettingsContent } from './SettingsStyles';
import { SmallButton, SmallButtonColor, SmallButtonGroup } from './SmallButton';

const StyledContextMenuButton = styled(Cell.Icon)({
  marginRight: '8px',
});

const StyledTitleInfoButton = styled(InfoButton)({
  marginLeft: '12px',
});

const StyledMethodInfoButton = styled(InfoButton)({
  marginRight: '11px',
});

const StyledSpinner = styled(ImageView)({
  height: '10px',
  width: '10px',
  marginRight: '6px',
});

const StyledNameLabel = styled(Cell.Label)({
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
});

const StyledTestResultCircle = styled.div<{ $result: boolean }>((props) => ({
  width: '10px',
  height: '10px',
  borderRadius: '50%',
  backgroundColor: props.$result ? colors.green : colors.red,
  marginRight: '6px',
}));

// This component is the topmost component in the API access methods view.
export default function ApiAccessMethods() {
  const history = useHistory();
  const methods = useSelector((state) => state.settings.apiAccessMethods);
  const currentMethod = useSelector((state) => state.settings.currentApiAccessMethod);

  return (
    <BackAction action={history.pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <NavigationBar>
              <NavigationItems>
                <TitleBarItem>
                  {
                    // TRANSLATORS: Title label in navigation bar
                    messages.pgettext('navigation-bar', 'API access')
                  }
                </TitleBarItem>
              </NavigationItems>
            </NavigationBar>

            <StyledNavigationScrollbars fillContainer>
              <StyledContent>
                <SettingsHeader>
                  <HeaderTitle>
                    {messages.pgettext('navigation-bar', 'API access')}
                    <StyledTitleInfoButton
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
                  </HeaderTitle>
                  <HeaderSubTitle>
                    {messages.pgettext(
                      'api-access-methods-view',
                      'Manage and add custom methods to access the Mullvad API.',
                    )}
                  </HeaderSubTitle>
                </SettingsHeader>

                <StyledSettingsContent>
                  <Cell.Group>
                    {methods.map((method) => (
                      <ApiAccessMethod
                        key={method.id}
                        method={method}
                        inUse={method.id === currentMethod?.id}
                      />
                    ))}
                  </Cell.Group>

                  <SmallButtonGroup $noMarginTop>
                    <SmallButton>{messages.pgettext('api-access-methods-view', 'Add')}</SmallButton>
                  </SmallButtonGroup>
                </StyledSettingsContent>
              </StyledContent>
            </StyledNavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

interface ApiAccessMethodProps {
  method: AccessMethodSetting;
  inUse: boolean;
}

function ApiAccessMethod(props: ApiAccessMethodProps) {
  const {
    setApiAccessMethod: setApiAccessMethodImpl,
    updateApiAccessMethod,
    removeApiAccessMethod,
  } = useAppContext();
  const history = useHistory();

  const [testing, testResult, testApiAccessMethod] = useApiAccessMethodTest();

  // State for delete confirmation dialog.
  const [removeConfirmationVisible, showRemoveConfirmation, hideRemoveConfirmation] = useBoolean();
  const confirmRemove = useCallback(() => {
    void removeApiAccessMethod(props.method.id);
    hideRemoveConfirmation();
  }, [props.method.id]);

  // Toggle on/off on an access method.
  const toggle = useCallback(
    async (value: boolean) => {
      const updatedMethod = cloneMethod(props.method);
      updatedMethod.enabled = value;
      await updateApiAccessMethod(updatedMethod);
    },
    [props.method],
  );

  const setApiAccessMethod = useCallback(async () => {
    const reachable = await testApiAccessMethod(props.method.id);
    if (reachable) {
      await setApiAccessMethodImpl(props.method.id);
    }
  }, [testApiAccessMethod, props.method.id]);

  const menuItems = useMemo<Array<ContextMenuItem>>(
    () => [
      {
        type: 'item' as const,
        label: 'Use',
        disabled: props.inUse,
        onClick: setApiAccessMethod,
      },
      { type: 'item' as const, label: 'Test', onClick: () => testApiAccessMethod(props.method.id) },
      // Edit and Delete shouldn't be available for direct and bridges.
      ...(props.method.type === 'direct' || props.method.type === 'bridges'
        ? []
        : [
            { type: 'separator' as const },
            {
              type: 'item' as const,
              label: 'Edit',
              onClick: () =>
                history.push(
                  generateRoutePath(RoutePath.editApiAccessMethods, { id: props.method.id }),
                ),
            },
            {
              type: 'item' as const,
              label: 'Delete',
              onClick: showRemoveConfirmation,
            },
          ]),
    ],
    [props.method.id, props.inUse, setApiAccessMethod, testApiAccessMethod, history.push],
  );

  return (
    <Cell.Row>
      <Cell.LabelContainer>
        <StyledNameLabel>{props.method.name}</StyledNameLabel>
        {testing && (
          <Cell.SubLabel>
            <StyledSpinner source="icon-spinner" />
            {messages.pgettext('api-access-methods-view', 'Testing...')}
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
      {props.method.type === 'direct' && (
        <StyledMethodInfoButton
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
        <StyledMethodInfoButton
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
      <ContextMenuContainer>
        <ContextMenuTrigger>
          <StyledContextMenuButton
            source="icon-more"
            tintColor={colors.white}
            tintHoverColor={colors.white80}
          />
        </ContextMenuTrigger>
        <ContextMenu items={menuItems} align="right" />
      </ContextMenuContainer>
      <Cell.Switch isOn={props.method.enabled} onChange={toggle} />

      {/* Confirmation dialog for method removal */}
      <ModalAlert
        isOpen={removeConfirmationVisible}
        type={ModalAlertType.warning}
        gridButtons={[
          <SmallButton key="cancel" onClick={hideRemoveConfirmation}>
            {messages.gettext('Cancel')}
          </SmallButton>,
          <SmallButton key="confirm" onClick={confirmRemove} color={SmallButtonColor.red}>
            {messages.pgettext('in-app-notifications', 'Delete')}
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
