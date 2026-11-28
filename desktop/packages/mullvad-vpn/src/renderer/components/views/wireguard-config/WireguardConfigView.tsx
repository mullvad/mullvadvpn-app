import { useCallback, useState } from 'react';
import styled from 'styled-components';

import { ImportConfigError, ImportConfigResponse } from '../../../../shared/daemon-rpc-types';
import { messages } from '../../../../shared/gettext';
import { useAppContext } from '../../../context';
import { Button, Text } from '../../../lib/components';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { colors } from '../../../lib/foundations';
import { useHistory } from '../../../lib/history';
import { useSelector } from '../../../redux/store';
import { AppNavigationHeader } from '../..';
import ClipboardLabel from '../../ClipboardLabel';
import { BackAction } from '../../keyboard-navigation';
import { HeaderTitle } from '../../SettingsHeader';
import { StatusDialog } from '../../status-dialog';
import { LabelledRow } from './LabelledRow';

const StyledViewContainer = styled(View.Container)`
  height: 100%;
  justify-content: space-between;
`;

const StyledTextArea = styled.textarea({
  width: '100%',
  minHeight: '90px',
  padding: '10px',
  borderRadius: '4px',
  border: 'none',
  resize: 'none',
  fontFamily: 'monospace',
  fontSize: '12px',
  color: colors.blue,
});

type ImportStatus = { type: 'success' } | { type: 'failed'; error: ImportConfigError };

export function WireguardConfigView() {
  const history = useHistory();
  const {
    importWireguardConfigText,
    importWireguardConfigFile,
    clearWireguardIdentity,
    showOpenDialog,
  } = useAppContext();

  const identity = useSelector((state) => state.identity.identity);

  const [pasteOpen, setPasteOpen] = useState(false);
  const [configText, setConfigText] = useState('');
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState<ImportStatus>();
  const [removeDialogOpen, setRemoveDialogOpen] = useState(false);

  const handleResponse = useCallback((response: ImportConfigResponse) => {
    if (response.type === 'success') {
      // The config contains a private key, don't keep it around any longer than necessary.
      setConfigText('');
      setPasteOpen(false);
      setStatus({ type: 'success' });
    } else {
      setStatus({ type: 'failed', error: response });
    }
    setBusy(false);
  }, []);

  const importFile = useCallback(async () => {
    const file = await showOpenDialog({
      properties: ['openFile'],
      buttonLabel: messages.gettext('Import'),
      filters: [
        {
          name: messages.pgettext('import-config-view', 'WireGuard config'),
          extensions: ['conf', 'txt'],
        },
        { name: messages.pgettext('import-config-view', 'All files'), extensions: ['*'] },
      ],
    });
    const path = file.filePaths[0];
    if (file.canceled || path === undefined) {
      return;
    }

    setBusy(true);
    handleResponse(await importWireguardConfigFile(path));
  }, [handleResponse, importWireguardConfigFile, showOpenDialog]);

  const importText = useCallback(async () => {
    setBusy(true);
    handleResponse(await importWireguardConfigText(configText));
  }, [configText, handleResponse, importWireguardConfigText]);

  const togglePaste = useCallback(() => {
    setPasteOpen((open) => !open);
    setConfigText('');
    setStatus(undefined);
  }, []);

  const onConfigTextChange = useCallback((event: React.ChangeEvent<HTMLTextAreaElement>) => {
    setConfigText(event.target.value);
  }, []);

  const openRemoveDialog = useCallback(() => setRemoveDialogOpen(true), []);

  const confirmRemove = useCallback(async () => {
    setRemoveDialogOpen(false);
    await clearWireguardIdentity();
  }, [clearWireguardIdentity]);

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={history.pop}>
        <AppNavigationHeader
          title={
            // TRANSLATORS: Title label in navigation bar
            messages.pgettext('wireguard-config-view', 'WireGuard config')
          }
        />

        <View.Content>
          <StyledViewContainer flexDirection="column" horizontalMargin="medium">
            <FlexColumn gap="medium">
              <Text variant="titleBig">
                <HeaderTitle>
                  {messages.pgettext('wireguard-config-view', 'WireGuard config')}
                </HeaderTitle>
              </Text>

              <FlexColumn gap="large">
                <LabelledRow label={messages.pgettext('wireguard-config-view', 'Public key')}>
                  <Text variant="bodySmallSemibold">
                    <ClipboardLabel
                      value={identity?.publicKey ?? ''}
                      obscureValue={false}
                      data-testid="wireguard-public-key"
                      copyLabel={messages.pgettext(
                        'accessibility',
                        'Copy the public key to the clipboard',
                      )}
                    />
                  </Text>
                </LabelledRow>

                <LabelledRow label={messages.pgettext('wireguard-config-view', 'IPv4 address')}>
                  <Text variant="bodySmallSemibold" data-testid="wireguard-ipv4">
                    {identity?.ipv4Address}
                  </Text>
                </LabelledRow>

                {identity?.ipv6Address !== undefined && (
                  <LabelledRow label={messages.pgettext('wireguard-config-view', 'IPv6 address')}>
                    <Text variant="bodySmallSemibold" data-testid="wireguard-ipv6">
                      {identity.ipv6Address}
                    </Text>
                  </LabelledRow>
                )}
              </FlexColumn>
            </FlexColumn>

            <FlexColumn gap="medium">
              {status !== undefined && (
                <Text
                  variant="labelTiny"
                  color={status.type === 'success' ? 'green' : 'red'}
                  aria-live="polite"
                  data-testid="wireguard-config-status">
                  {statusString(status)}
                </Text>
              )}

              {pasteOpen && (
                <FlexColumn gap="small">
                  <StyledTextArea
                    value={configText}
                    onChange={onConfigTextChange}
                    disabled={busy}
                    spellCheck={false}
                    autoCapitalize="off"
                    autoCorrect="off"
                    data-testid="wireguard-config-text"
                    aria-label={messages.pgettext(
                      'import-config-view',
                      'Paste the contents of the WireGuard config here',
                    )}
                  />
                  <Button
                    color="success"
                    onClick={importText}
                    disabled={busy || configText.trim() === ''}>
                    <Button.Text>
                      {messages.pgettext('import-config-view', 'Import pasted config')}
                    </Button.Text>
                  </Button>
                </FlexColumn>
              )}

              <Button onClick={importFile} disabled={busy}>
                <Button.Text>
                  {
                    // TRANSLATORS: Button label for replacing the current WireGuard config with the
                    // TRANSLATORS: contents of a file.
                    messages.pgettext('wireguard-config-view', 'Replace with config file')
                  }
                </Button.Text>
              </Button>

              <Button onClick={togglePaste} disabled={busy}>
                <Button.Text>
                  {pasteOpen
                    ? messages.gettext('Cancel')
                    : // TRANSLATORS: Button label for replacing the current WireGuard config with
                      // TRANSLATORS: pasted text.
                      messages.pgettext('wireguard-config-view', 'Replace with pasted config')}
                </Button.Text>
              </Button>

              <Button color="destructive" onClick={openRemoveDialog} disabled={busy}>
                <Button.Text>
                  {
                    // TRANSLATORS: Button label for removing the imported WireGuard config.
                    messages.pgettext('wireguard-config-view', 'Remove config')
                  }
                </Button.Text>
              </Button>
            </FlexColumn>

            <StatusDialog
              variant="warning"
              open={removeDialogOpen}
              onOpenChange={setRemoveDialogOpen}>
              <StatusDialog.Title>
                {messages.pgettext('wireguard-config-view', 'Remove the WireGuard config?')}
              </StatusDialog.Title>
              <StatusDialog.Text>
                {messages.pgettext(
                  'wireguard-config-view',
                  'The private key is deleted from this computer and the VPN is disconnected. You will have to import a config again to connect.',
                )}
              </StatusDialog.Text>
              <StatusDialog.ButtonGroup>
                <StatusDialog.Button onClick={confirmRemove} color="destructive">
                  <StatusDialog.Button.Text>
                    {messages.pgettext('wireguard-config-view', 'Remove')}
                  </StatusDialog.Button.Text>
                </StatusDialog.Button>
                <StatusDialog.CloseButton>
                  <StatusDialog.CloseButton.Text>
                    {messages.gettext('Cancel')}
                  </StatusDialog.CloseButton.Text>
                </StatusDialog.CloseButton>
              </StatusDialog.ButtonGroup>
            </StatusDialog>
          </StyledViewContainer>
        </View.Content>
      </BackAction>
    </View>
  );
}

function statusString(status: ImportStatus): string {
  if (status.type === 'success') {
    return messages.pgettext('wireguard-config-view', 'The config was replaced');
  }

  switch (status.error.error) {
    case 'invalid-config':
      return (
        status.error.message ??
        messages.pgettext('import-config-view', 'This is not a valid WireGuard config')
      );
    case 'file-read':
      return messages.pgettext('import-config-view', 'The file could not be read');
    case 'communication':
      return messages.pgettext('import-config-view', 'Could not reach the Mullvad VPN service');
  }
}
