import { createContext, ReactNode, useCallback, useContext, useMemo, useState } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { urls } from '../../shared/constants';
import { messages } from '../../shared/gettext';
import log from '../../shared/logging';
import { RoutePath } from '../../shared/routes';
import { useAppContext } from '../context';
import { LockdownModeSwitch } from '../features/tunnel/components';
import { Button, Flex } from '../lib/components';
import { FlexColumn } from '../lib/components/flex-column';
import { spacings } from '../lib/foundations';
import { useHistory } from '../lib/history';
import { useExclusiveTask } from '../lib/hooks/use-exclusive-task';
import { IconBadge } from '../lib/icon-badge';
import { formatDeviceName } from '../lib/utils';
import { useSelector } from '../redux/store';
import { AppMainHeader } from './app-main-header';
import DeviceInfoButton from './DeviceInfoButton';
import {
  StyledAccountNumberContainer,
  StyledAccountNumberLabel,
  StyledAccountNumberMessage,
  StyledBody,
  StyledContainer,
  StyledCustomScrollbars,
  StyledDeviceLabel,
  StyledMessage,
  StyledTitle,
} from './ExpiredAccountErrorViewStyles';
import { Footer, Layout } from './Layout';
import { ModalAlert, ModalAlertType, ModalMessage } from './Modal';
import { SettingsListItem } from './settings-list-item';

enum RecoveryAction {
  openBrowser,
  disconnect,
  disableLockdownMode,
}

const StyledSettingsToggleListItem = styled(SettingsListItem)`
  margin-top: ${spacings.medium};
`;

export default function ExpiredAccountErrorView() {
  return (
    <ExpiredAccountContextProvider>
      <ExpiredAccountErrorViewComponent />
    </ExpiredAccountContextProvider>
  );
}

function ExpiredAccountErrorViewComponent() {
  const { push } = useHistory();
  const { disconnectTunnel } = useAppContext();

  const { recoveryAction } = useRecoveryAction();
  const isNewAccount = useIsNewAccount();

  const [disconnect, disconnecting] = useExclusiveTask(async () => {
    try {
      await disconnectTunnel();
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to disconnect the tunnel: ${error.message}`);
    }
  });

  const navigateToRedeemVoucher = useCallback(() => {
    push(RoutePath.redeemVoucher);
  }, [push]);

  return (
    <Layout>
      <AppMainHeader
        variant={isNewAccount ? 'default' : 'basedOnConnectionStatus'}
        size="basedOnLoginStatus">
        <AppMainHeader.AccountButton />
        <AppMainHeader.SettingsButton />
      </AppMainHeader>
      <StyledCustomScrollbars fillContainer>
        <StyledContainer>
          <StyledBody>{isNewAccount ? <WelcomeView /> : <Content />}</StyledBody>

          <Footer>
            <FlexColumn $gap="medium">
              {recoveryAction === RecoveryAction.disconnect && (
                <Button variant="destructive" disabled={disconnecting} onClick={disconnect}>
                  <Button.Text>
                    {
                      // TRANSLATORS: Button label for disconnecting from the VPN.
                      messages.pgettext('connect-view', 'Disconnect')
                    }
                  </Button.Text>
                </Button>
              )}

              <ExternalPaymentButton />

              <Button variant="success" onClick={navigateToRedeemVoucher}>
                <Button.Text>
                  {
                    // TRANSLATORS: Button label for navigating to the voucher redemption view.
                    messages.pgettext('connect-view', 'Redeem voucher')
                  }
                </Button.Text>
              </Button>
            </FlexColumn>
          </Footer>

          <LockdownModeAlert />
        </StyledContainer>
      </StyledCustomScrollbars>
    </Layout>
  );
}

function WelcomeView() {
  const account = useSelector((state) => state.account);
  const { recoveryMessage } = useRecoveryAction();

  return (
    <>
      <StyledTitle data-testid="title">
        {messages.pgettext('connect-view', 'Congrats!')}
      </StyledTitle>
      <StyledAccountNumberMessage>
        {messages.pgettext('connect-view', 'Here’s your account number. Save it!')}
        <StyledAccountNumberContainer>
          <StyledAccountNumberLabel
            accountNumber={account.accountNumber || ''}
            obscureValue={false}
          />
        </StyledAccountNumberContainer>
      </StyledAccountNumberMessage>

      <Flex $alignItems="center" $gap="tiny" $margin={{ bottom: 'medium' }}>
        <StyledDeviceLabel>
          {sprintf(
            // TRANSLATORS: A label that will display the newly created device name to inform the user
            // TRANSLATORS: about it.
            // TRANSLATORS: Available placeholders:
            // TRANSLATORS: %(deviceName)s - The name of the current device
            messages.pgettext('device-management', 'Device name: %(deviceName)s'),
            {
              deviceName: formatDeviceName(account.deviceName ?? ''),
            },
          )}
        </StyledDeviceLabel>
        <DeviceInfoButton />
      </Flex>

      <StyledMessage>
        {sprintf('%(introduction)s %(recoveryMessage)s', {
          introduction: messages.pgettext(
            'connect-view',
            'To start using the app, you first need to add time to your account.',
          ),
          recoveryMessage,
        })}
      </StyledMessage>
    </>
  );
}

function Content() {
  const { recoveryMessage } = useRecoveryAction();

  return (
    <>
      <Flex $justifyContent="center" $margin={{ bottom: 'medium' }}>
        <IconBadge state="negative" />
      </Flex>
      <StyledTitle data-testid="title">
        {messages.pgettext('connect-view', 'Out of time')}
      </StyledTitle>
      <StyledMessage>
        {sprintf('%(introduction)s %(recoveryMessage)s', {
          introduction: messages.pgettext(
            'connect-view',
            'You have no more VPN time left on this account.',
          ),
          recoveryMessage,
        })}
      </StyledMessage>
    </>
  );
}

function ExternalPaymentButton() {
  const { setShowLockdownModeAlert } = useExpiredAccountContext();
  const { recoveryAction } = useRecoveryAction();
  const { openUrlWithAuth } = useAppContext();
  const isNewAccount = useIsNewAccount();

  const buttonText = isNewAccount
    ? messages.gettext('Buy credit')
    : messages.gettext('Buy more credit');

  const [openExternalPayment, openingExternalPayment] = useExclusiveTask(async () => {
    if (recoveryAction === RecoveryAction.disableLockdownMode) {
      setShowLockdownModeAlert(true);
    } else {
      await openUrlWithAuth(urls.purchase);
    }
  });

  return (
    <Button
      variant="success"
      disabled={openingExternalPayment || recoveryAction === RecoveryAction.disconnect}
      onClick={openExternalPayment}
      aria-description={
        // TRANSLATORS: Accessibility label for the button that opens the browser to buy credit.
        messages.pgettext('accessibility', 'Opens externally')
      }>
      <Button.Text>{buttonText}</Button.Text>
      <Button.Icon icon="external" />
    </Button>
  );
}

function LockdownModeAlert() {
  const { showLockdownModeAlert, setShowLockdownModeAlert } = useExpiredAccountContext();

  const onCloseLockdownModeInstructions = useCallback(() => {
    setShowLockdownModeAlert(false);
  }, [setShowLockdownModeAlert]);

  return (
    <ModalAlert
      isOpen={showLockdownModeAlert}
      type={ModalAlertType.caution}
      buttons={[
        <Button key="cancel" onClick={onCloseLockdownModeInstructions}>
          <Button.Text>{messages.gettext('Close')}</Button.Text>
        </Button>,
      ]}
      close={onCloseLockdownModeInstructions}>
      <ModalMessage>
        {messages.pgettext(
          'connect-view',
          'You need to disable "Lockdown mode" in order to access the Internet to add time.',
        )}
      </ModalMessage>
      <ModalMessage>
        {messages.pgettext(
          'connect-view',
          'Remember, turning it off will allow network traffic while the VPN is disconnected until you turn it back on under Advanced settings.',
        )}
      </ModalMessage>
      <StyledSettingsToggleListItem>
        <SettingsListItem.Item>
          <SettingsListItem.Content>
            <LockdownModeSwitch>
              <LockdownModeSwitch.Label variant="titleMedium">
                {messages.pgettext('vpn-settings-view', 'Lockdown mode')}
              </LockdownModeSwitch.Label>
              <LockdownModeSwitch.Trigger>
                <LockdownModeSwitch.Thumb />
              </LockdownModeSwitch.Trigger>
            </LockdownModeSwitch>
          </SettingsListItem.Content>
        </SettingsListItem.Item>
      </StyledSettingsToggleListItem>
    </ModalAlert>
  );
}

type ExpiredAccountContextType = {
  setShowLockdownModeAlert: (val: boolean) => void;
  showLockdownModeAlert: boolean;
};

const ExpiredAccountContext = createContext<ExpiredAccountContextType | undefined>(undefined);

const ExpiredAccountContextProvider = ({ children }: { children: ReactNode }) => {
  const [showLockdownModeAlert, setShowLockdownModeAlert] = useState(false);

  const value: ExpiredAccountContextType = useMemo(
    () => ({
      setShowLockdownModeAlert,
      showLockdownModeAlert,
    }),
    [setShowLockdownModeAlert, showLockdownModeAlert],
  );
  return <ExpiredAccountContext.Provider value={value}>{children}</ExpiredAccountContext.Provider>;
};

const useExpiredAccountContext = () => {
  const context = useContext(ExpiredAccountContext);
  if (!context) {
    throw new Error(
      'useExpiredAccountContext must be used within an ExpiredAccountContextProvider',
    );
  }

  return context;
};

const useRecoveryAction = () => {
  const isBlocked = useSelector((state) => state.connection.isBlocked);
  const lockdownMode = useSelector((state) => state.settings.lockdownMode);

  let recoveryAction: RecoveryAction;

  if (lockdownMode && isBlocked) {
    recoveryAction = RecoveryAction.disableLockdownMode;
  } else if (!lockdownMode && isBlocked) {
    recoveryAction = RecoveryAction.disconnect;
  } else {
    recoveryAction = RecoveryAction.openBrowser;
  }

  let recoveryMessage: string;

  switch (recoveryAction) {
    case RecoveryAction.openBrowser:
    case RecoveryAction.disableLockdownMode:
      recoveryMessage = messages.pgettext(
        'connect-view',
        'Either buy credit on our website or redeem a voucher.',
      );
      break;
    case RecoveryAction.disconnect:
      recoveryMessage = messages.pgettext(
        'connect-view',
        'To add more, you will need to disconnect and access the Internet with an unsecure connection.',
      );
      break;
  }

  return { recoveryAction, recoveryMessage };
};

const useIsNewAccount = () => {
  const account = useSelector((state) => state.account);
  return account.status.type === 'ok' && account.status.method === 'new_account';
};
