import React, { useCallback, useEffect, useState } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import { RoutePath } from '../../../../shared/routes';
import { useScheduler } from '../../../../shared/scheduler';
import { useAppContext } from '../../../context';
import useActions from '../../../lib/actionsHook';
import { Button, Icon, IconProps, LabelTinySemiBold } from '../../../lib/components';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { colors, spacings } from '../../../lib/foundations';
import { TransitionType, useHistory } from '../../../lib/history';
import { useEffectEvent } from '../../../lib/utility-hooks';
import settingsImportActions from '../../../redux/settings-import/actions';
import { useSelector } from '../../../redux/store';
import { AppNavigationHeader } from '../..';
import { ButtonGroup } from '../../ButtonGroup';
import { normalText } from '../../common-styles';
import { BackAction } from '../../keyboard-navigation';
import { HeaderSubTitle, HeaderTitle } from '../../SettingsHeader';
import { StatusDialog } from '../../status-dialog';

type ImportStatus = { successful: boolean } & ({ type: 'file'; name: string } | { type: 'text' });

export function SettingsImportView() {
  const history = useHistory();
  const {
    clearAllRelayOverrides,
    importSettingsFile,
    importSettingsText,
    showOpenDialog,
    getPathBaseName,
  } = useAppContext();
  const { clearSettingsImportForm, unsetSubmitSettingsImportForm } =
    useActions(settingsImportActions);

  // Status of the text form which is used to for example submit it.
  const textForm = useSelector((state) => state.settingsImport);

  // "Clear" button will be disabled if there are no imported overrides.
  const activeOverrides = useSelector((state) => state.settings.relayOverrides.length > 0);

  const [clearDialogOpen, setClearDialogOpen] = React.useState(false);

  // Keeps the status of the last import and is cleared 10 seconds after being set.
  const [importStatus, setImportStatusImpl] = useState<ImportStatus>();
  const importStatusResetScheduler = useScheduler();

  const setImportStatus = useCallback(
    (status?: ImportStatus) => {
      // Cancel scheduled status clearing.
      importStatusResetScheduler.cancel();
      setImportStatusImpl(status);

      // The status text should be cleared after 10 seconds.
      if (status !== undefined) {
        importStatusResetScheduler.schedule(() => setImportStatusImpl(undefined), 10_000);
      }
    },
    [importStatusResetScheduler],
  );

  const confirmClear = useCallback(() => {
    setClearDialogOpen(false);
    void clearAllRelayOverrides();
    setImportStatus(undefined);
  }, [clearAllRelayOverrides, setImportStatus]);

  const handleClickClearAllOverrides = useCallback(() => {
    setClearDialogOpen(true);
  }, []);

  const navigateTextImport = useCallback(() => {
    history.push(RoutePath.settingsTextImport, { transition: TransitionType.show });
  }, [history]);

  const importFile = useCallback(async () => {
    const file = await showOpenDialog({
      properties: ['openFile'],
      buttonLabel: messages.gettext('Import'),
      filters: [{ name: 'Mullvad settings file', extensions: ['json'] }],
    });
    const path = file.filePaths[0];
    const name = await getPathBaseName(path);
    try {
      await importSettingsFile(path);
      setImportStatus({ successful: true, type: 'file', name });
    } catch {
      setImportStatus({ successful: false, type: 'file', name });
    }
  }, [getPathBaseName, importSettingsFile, setImportStatus, showOpenDialog]);

  const onMount = useEffectEvent(async () => {
    if (history.action === 'POP' && textForm.submit && textForm.value !== '') {
      try {
        await importSettingsText(textForm.value);
        setImportStatus({ successful: true, type: 'text' });
        clearSettingsImportForm();
      } catch {
        setImportStatus({ successful: false, type: 'text' });
        unsetSubmitSettingsImportForm();
      }
    }
  });

  // These lint rules are disabled for now because the react plugin for eslint does
  // not understand that useEffectEvent should not be added to the dependency array.
  // Enable these rules again when eslint can lint useEffectEvent properly.
  // eslint-disable-next-line react-compiler/react-compiler
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(() => void onMount(), []);

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={history.pop}>
        <AppNavigationHeader
          title={
            // TRANSLATORS: Title label in navigation bar. This is for a feature that lets
            // TRANSLATORS: users import server IP settings.
            messages.pgettext('settings-import', 'Server IP override')
          }>
          <AppNavigationHeader.Info>
            <AppNavigationHeader.Info.Button />
            <AppNavigationHeader.Info.Dialog>
              <AppNavigationHeader.Info.Dialog.Title>
                {messages.pgettext('settings-import', 'Server IP override')}
              </AppNavigationHeader.Info.Dialog.Title>
              <AppNavigationHeader.Info.Dialog.Text>
                {messages.pgettext(
                  'settings-import',
                  'On some networks, where various types of censorship are being used, our server IP addresses are sometimes blocked.',
                )}
              </AppNavigationHeader.Info.Dialog.Text>
              <AppNavigationHeader.Info.Dialog.Text>
                {messages.pgettext(
                  'settings-import',
                  'To circumvent this you can import a file or a text, provided by our support team, with new IP addresses that override the default addresses of the servers in the Select location view.',
                )}
              </AppNavigationHeader.Info.Dialog.Text>
              <AppNavigationHeader.Info.Dialog.Text>
                {messages.pgettext(
                  'settings-import',
                  'If you are having issues connecting to VPN servers, please contact support.',
                )}
              </AppNavigationHeader.Info.Dialog.Text>
            </AppNavigationHeader.Info.Dialog>
          </AppNavigationHeader.Info>
        </AppNavigationHeader>

        <View.Content>
          <View.Container
            horizontalMargin="medium"
            flexDirection="column"
            flexGrow={1}
            justifyContent="space-between">
            <FlexColumn gap="medium">
              <FlexColumn gap="small">
                <HeaderTitle>
                  {messages.pgettext('settings-import', 'Server IP override')}
                </HeaderTitle>
                <HeaderSubTitle>
                  {messages.pgettext(
                    'settings-import',
                    'Import files or text with new IP addresses for the servers in the Select location view.',
                  )}
                </HeaderSubTitle>
              </FlexColumn>
              <View.Container horizontalMargin="small" flexDirection="column" gap="large">
                <ButtonGroup gap="medium">
                  <Button onClick={navigateTextImport}>
                    <Button.Text>
                      {messages.pgettext('settings-import', 'Import via text')}
                    </Button.Text>
                  </Button>
                  <Button onClick={importFile}>
                    <Button.Text>{messages.pgettext('settings-import', 'Import file')}</Button.Text>
                  </Button>
                </ButtonGroup>

                <SettingsImportStatus status={importStatus} />
              </View.Container>
            </FlexColumn>
            <Button
              variant="destructive"
              onClick={handleClickClearAllOverrides}
              disabled={!activeOverrides}>
              <Button.Text>
                {messages.pgettext('settings-import', 'Clear all overrides')}
              </Button.Text>
            </Button>
            <StatusDialog
              variant="warning"
              open={clearDialogOpen}
              onOpenChange={setClearDialogOpen}>
              <StatusDialog.Title>
                {messages.pgettext('settings-import', 'Clear all overrides?')}
              </StatusDialog.Title>
              <StatusDialog.Text>
                {messages.pgettext(
                  'settings-import',
                  'Clearing the imported overrides changes the server IPs, in the Select location view, back to default.',
                )}
              </StatusDialog.Text>
              <StatusDialog.ButtonGroup>
                <StatusDialog.Button onClick={confirmClear} variant="destructive">
                  <StatusDialog.Button.Text>{messages.gettext('Clear')}</StatusDialog.Button.Text>
                </StatusDialog.Button>
                <StatusDialog.CloseButton>
                  <StatusDialog.CloseButton.Text>
                    {messages.gettext('Cancel')}
                  </StatusDialog.CloseButton.Text>
                </StatusDialog.CloseButton>
              </StatusDialog.ButtonGroup>
            </StatusDialog>
          </View.Container>
        </View.Content>
      </BackAction>
    </View>
  );
}

const StyledStatusTitle = styled.div(normalText, {
  display: 'flex',
  alignItems: 'center',
  fontWeight: 'bold',
  lineHeight: '20px',
  color: colors.white,
  gap: spacings.tiny,
});

interface ImportStatusProps {
  status?: ImportStatus;
}

// This component renders the status title, subtitle and icon depending on active overrides and
// import result.
function SettingsImportStatus(props: ImportStatusProps) {
  const activeOverrides = useSelector((state) => state.settings.relayOverrides.length > 0);

  let title;
  if (props.status?.successful) {
    title = messages.pgettext('settings-import', 'IMPORT SUCCESSFUL');
  } else if (activeOverrides && props.status?.successful !== false) {
    title = messages.pgettext('settings-import', 'OVERRIDES ACTIVE');
  } else {
    title = messages.pgettext('settings-import', 'NO OVERRIDES IMPORTED');
  }

  let iconProps: Pick<IconProps, 'icon' | 'color'> | undefined = undefined;
  let subtitle;
  if (props.status !== undefined) {
    iconProps = props.status.successful
      ? {
          icon: 'checkmark',
          color: 'green',
        }
      : { icon: 'cross', color: 'red' };

    if (props.status.successful) {
      subtitle =
        props.status.type === 'file'
          ? sprintf(
              messages.pgettext(
                'settings-import',
                'Import of file %(fileName)s was successful, overrides are now active.',
              ),
              { fileName: props.status.name },
            )
          : messages.pgettext(
              'settings-import',
              'Import of text was successful, overrides are now active.',
            );
    } else {
      subtitle =
        props.status.type === 'file'
          ? sprintf(
              messages.pgettext(
                'settings-import',
                'Import of file %(fileName)s was unsuccessful, please try again.',
              ),
              { fileName: props.status.name },
            )
          : messages.pgettext(
              'settings-import',
              'Import of text was unsuccessful, please try again.',
            );
    }
  }

  return (
    <FlexColumn>
      <StyledStatusTitle data-testid="status-title">
        {title}
        {iconProps !== undefined && <Icon {...iconProps} size="medium" />}
      </StyledStatusTitle>
      {subtitle !== undefined && (
        <LabelTinySemiBold data-testid="status-subtitle" color="whiteAlpha60">
          {subtitle}
        </LabelTinySemiBold>
      )}
    </FlexColumn>
  );
}
