import { useCallback } from 'react';

import { CustomProxy } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { Button } from '../lib/components';
import { useBridgeSettingsUpdater } from '../lib/constraint-updater';
import { useHistory } from '../lib/history';
import { useBoolean } from '../lib/utility-hooks';
import { useSelector } from '../redux/store';
import { AppNavigationHeader } from './';
import { SettingsForm } from './cell/SettingsForm';
import { BackAction } from './keyboard-navigation';
import { Layout, SettingsContainer, SettingsContent, SettingsNavigationScrollbars } from './Layout';
import { ModalAlert, ModalAlertType } from './Modal';
import { NavigationContainer } from './NavigationContainer';
import { ProxyForm } from './ProxyForm';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';

export function EditCustomBridge() {
  return (
    <SettingsForm>
      <CustomBridgeForm />
    </SettingsForm>
  );
}

function CustomBridgeForm() {
  const { pop } = useHistory();
  const bridgeSettingsUpdater = useBridgeSettingsUpdater();
  const bridgeSettings = useSelector((state) => state.settings.bridgeSettings);

  const [deleteDialogVisible, showDeleteDialog, hideDeleteDialog] = useBoolean();

  // If there are no custom bridge settings, we should prompt the user to add a custom bridge.
  // Otherwise, we should prompt them to edit the existing custom bridge settings.
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
      pop();
    },
    [bridgeSettingsUpdater, pop],
  );

  const onDelete = useCallback(() => {
    if (bridgeSettings.custom !== undefined) {
      hideDeleteDialog();
      void bridgeSettingsUpdater((bridgeSettings) => {
        bridgeSettings.type = 'normal';
        delete bridgeSettings.custom;
        return bridgeSettings;
      });
      pop();
    }
  }, [bridgeSettings.custom, bridgeSettingsUpdater, hideDeleteDialog, pop]);

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader title={title} />

            <SettingsNavigationScrollbars fillContainer>
              <SettingsContent>
                <SettingsHeader>
                  <HeaderTitle>{title}</HeaderTitle>
                </SettingsHeader>

                <ProxyForm
                  proxy={bridgeSettings.custom}
                  onSave={onSave}
                  onCancel={pop}
                  onDelete={bridgeSettings.custom === undefined ? undefined : showDeleteDialog}
                />

                <ModalAlert
                  isOpen={deleteDialogVisible}
                  type={ModalAlertType.warning}
                  gridButtons={[
                    <Button key="cancel" onClick={hideDeleteDialog}>
                      <Button.Text>{messages.gettext('Cancel')}</Button.Text>
                    </Button>,
                    <Button
                      key="delete"
                      variant="destructive"
                      onClick={onDelete}
                      data-testid="delete-confirm">
                      <Button.Text>{messages.gettext('Delete')}</Button.Text>
                    </Button>,
                  ]}
                  close={hideDeleteDialog}
                  title={messages.pgettext('custom-bridge', 'Delete custom bridge?')}
                  message={messages.pgettext(
                    'custom-bridge',
                    'Deleting the custom bridge will take you back to the select location view and the Automatic option will be selected instead.',
                  )}
                />
              </SettingsContent>
            </SettingsNavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}
