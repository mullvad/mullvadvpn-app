import { useCallback } from 'react';

import { CustomProxy } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { useBridgeSettingsUpdater } from '../lib/constraint-updater';
import { useHistory } from '../lib/history';
import { useBoolean } from '../lib/utilityHooks';
import { useSelector } from '../redux/store';
import { SettingsForm } from './cell/SettingsForm';
import { BackAction } from './KeyboardNavigation';
import { Layout, SettingsContainer } from './Layout';
import { ModalAlert, ModalAlertType } from './Modal';
import { NavigationBar, NavigationContainer, NavigationItems, TitleBarItem } from './NavigationBar';
import { ProxyForm } from './ProxyForm';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';
import { StyledContent, StyledNavigationScrollbars, StyledSettingsContent } from './SettingsStyles';
import { SmallButton } from './SmallButton';
import { SmallButtonColor } from './SmallButton';

export function EditCustomBridge() {
  return (
    <SettingsForm>
      <CustomBridgeForm></CustomBridgeForm>
    </SettingsForm>
  );
}

function CustomBridgeForm() {
  const history = useHistory();
  const bridgeSettingsUpdater = useBridgeSettingsUpdater();
  const bridgeSettings = useSelector((state) => state.settings.bridgeSettings);

  const [deleteDialogVisible, showDeleteDialog, hideDeleteDialog] = useBoolean();

  const title =
    bridgeSettings.custom === undefined
      ? messages.pgettext('custom-bridge', 'Add custom bridge')
      : messages.pgettext('custom-bridge', 'Edit custom bridge');

  const onSave = useCallback(
    (newBridge: CustomProxy) => {
      void bridgeSettingsUpdater((bridgeSettings) => {
        bridgeSettings.type = 'custom';
        bridgeSettings.custom = newBridge;
        return bridgeSettings;
      });
      history.pop();
    },
    [bridgeSettingsUpdater, history.pop],
  );

  const onDelete = useCallback(() => {
    if (bridgeSettings.custom !== undefined) {
      hideDeleteDialog();
      void bridgeSettingsUpdater((bridgeSettings) => {
        bridgeSettings.type = 'normal';
        delete bridgeSettings.custom;
        return bridgeSettings;
      });
      history.pop();
    }
  }, [bridgeSettingsUpdater, history.pop]);

  return (
    <BackAction action={history.pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <NavigationBar>
              <NavigationItems>
                <TitleBarItem>{title}</TitleBarItem>
              </NavigationItems>
            </NavigationBar>

            <StyledNavigationScrollbars fillContainer>
              <StyledContent>
                <SettingsHeader>
                  <HeaderTitle>{title}</HeaderTitle>
                </SettingsHeader>

                <StyledSettingsContent>
                  <ProxyForm
                    proxy={bridgeSettings.custom}
                    onSave={onSave}
                    onCancel={history.pop}
                    onDelete={bridgeSettings.custom === undefined ? undefined : showDeleteDialog}
                  />
                </StyledSettingsContent>

                <ModalAlert
                  isOpen={deleteDialogVisible}
                  type={ModalAlertType.warning}
                  gridButtons={[
                    <SmallButton key="cancel" onClick={hideDeleteDialog}>
                      {messages.gettext('Cancel')}
                    </SmallButton>,
                    <SmallButton key="delete" color={SmallButtonColor.red} onClick={onDelete}>
                      {messages.gettext('Delete')}
                    </SmallButton>,
                  ]}
                  close={hideDeleteDialog}
                  title={messages.pgettext('custom-bridge', 'Delete custom bridge?')}
                  message={messages.pgettext(
                    'custom-bridge',
                    'Deleting the custom bridge will take you back to the select location view and the Automatic option will be selected instead.',
                  )}
                />
              </StyledContent>
            </StyledNavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}
