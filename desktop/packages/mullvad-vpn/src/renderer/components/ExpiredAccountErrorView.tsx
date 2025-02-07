import { createContext, ReactNode, useCallback, useContext, useMemo, useState } from 'react';
import { sprintf } from 'sprintf-js';

import { urls } from '../../shared/constants';
import { messages } from '../../shared/gettext';
import log from '../../shared/logging';
import { capitalizeEveryWord } from '../../shared/string-helpers';
import { useAppContext } from '../context';
import { Flex, Icon } from '../lib/components';
import { Spacings } from '../lib/foundations';
import { useHistory } from '../lib/history';
import { IconBadge } from '../lib/icon-badge';
import { RoutePath } from '../lib/routes';
import { useSelector } from '../redux/store';
import { AppMainHeader } from './app-main-header';
import * as AppButton from './AppButton';
import { AriaDescribed, AriaDescription, AriaDescriptionGroup } from './AriaGroup';
import * as Cell from './cell';
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
  StyledModalCellContainer,
  StyledTitle,
} from './ExpiredAccountErrorViewStyles';
import { Footer, Layout } from './Layout';
import { ModalAlert, ModalAlertType, ModalMessage } from './Modal';

enum RecoveryAction {
  openBrowser,
  disconnect,
  disableBlockedWhenDisconnected,
}

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

  const onDisconnect = useCallback(async () => {
    try {
      await disconnectTunnel();
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to disconnect the tunnel: ${error.message}`);
    }
  }, [disconnectTunnel]);

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
            <AppButton.ButtonGroup>
              {recoveryAction === RecoveryAction.disconnect && (
                <AppButton.BlockingButton onClick={onDisconnect}>
                  <AppButton.RedButton>
                    {messages.pgettext('connect-view', 'Disconnect')}
                  </AppButton.RedButton>
                </AppButton.BlockingButton>
              )}

              <ExternalPaymentButton />

              <AppButton.GreenButton onClick={navigateToRedeemVoucher}>
                {messages.pgettext('connect-view', 'Redeem voucher')}
              </AppButton.GreenButton>
            </AppButton.ButtonGroup>
          </Footer>

          <BlockWhenDisconnectedAlert />
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

      <Flex $alignItems="center" $gap={Spacings.spacing1} $margin={{ bottom: Spacings.spacing5 }}>
        <StyledDeviceLabel>
          {sprintf(
            // TRANSLATORS: A label that will display the newly created device name to inform the user
            // TRANSLATORS: about it.
            // TRANSLATORS: Available placeholders:
            // TRANSLATORS: %(deviceName)s - The name of the current device
            messages.pgettext('device-management', 'Device name: %(deviceName)s'),
            {
              deviceName: capitalizeEveryWord(account.deviceName ?? ''),
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
      <Flex $justifyContent="center" $margin={{ bottom: Spacings.spacing5 }}>
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
  const { setShowBlockWhenDisconnectedAlert } = useExpiredAccountContext();
  const { recoveryAction } = useRecoveryAction();
  const { openUrlWithAuth } = useAppContext();
  const isNewAccount = useIsNewAccount();

  const buttonText = isNewAccount
    ? messages.gettext('Buy credit')
    : messages.gettext('Buy more credit');

  const onOpenExternalPayment = useCallback(async () => {
    if (recoveryAction === RecoveryAction.disableBlockedWhenDisconnected) {
      setShowBlockWhenDisconnectedAlert(true);
    } else {
      await openUrlWithAuth(urls.purchase);
    }
  }, [openUrlWithAuth, recoveryAction, setShowBlockWhenDisconnectedAlert]);

  return (
    <AppButton.BlockingButton
      disabled={recoveryAction === RecoveryAction.disconnect}
      onClick={onOpenExternalPayment}>
      <AriaDescriptionGroup>
        <AriaDescribed>
          <AppButton.GreenButton>
            <AppButton.Label>{buttonText}</AppButton.Label>
            <AriaDescription>
              <Icon
                icon="external"
                aria-label={messages.pgettext('accessibility', 'Opens externally')}
              />
            </AriaDescription>
          </AppButton.GreenButton>
        </AriaDescribed>
      </AriaDescriptionGroup>
    </AppButton.BlockingButton>
  );
}

function BlockWhenDisconnectedAlert() {
  const { showBlockWhenDisconnectedAlert, setShowBlockWhenDisconnectedAlert } =
    useExpiredAccountContext();
  const { setBlockWhenDisconnected } = useAppContext();
  const blockWhenDisconnected = useSelector((state) => state.settings.blockWhenDisconnected);

  const onCloseBlockWhenDisconnectedInstructions = useCallback(() => {
    setShowBlockWhenDisconnectedAlert(false);
  }, [setShowBlockWhenDisconnectedAlert]);

  const onChange = useCallback(
    async (blockWhenDisconnected: boolean) => {
      try {
        await setBlockWhenDisconnected(blockWhenDisconnected);
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update block when disconnected', error.message);
      }
    },
    [setBlockWhenDisconnected],
  );

  return (
    <ModalAlert
      isOpen={showBlockWhenDisconnectedAlert}
      type={ModalAlertType.caution}
      buttons={[
        <AppButton.BlueButton key="cancel" onClick={onCloseBlockWhenDisconnectedInstructions}>
          {messages.gettext('Close')}
        </AppButton.BlueButton>,
      ]}
      close={onCloseBlockWhenDisconnectedInstructions}>
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
      <StyledModalCellContainer>
        <Cell.Label>{messages.pgettext('vpn-settings-view', 'Lockdown mode')}</Cell.Label>
        <Cell.Switch isOn={blockWhenDisconnected} onChange={onChange} />
      </StyledModalCellContainer>
    </ModalAlert>
  );
}

type ExpiredAccountContextType = {
  setShowBlockWhenDisconnectedAlert: (val: boolean) => void;
  showBlockWhenDisconnectedAlert: boolean;
};

const ExpiredAccountContext = createContext<ExpiredAccountContextType | undefined>(undefined);

const ExpiredAccountContextProvider = ({ children }: { children: ReactNode }) => {
  const [showBlockWhenDisconnectedAlert, setShowBlockWhenDisconnectedAlert] = useState(false);

  const value: ExpiredAccountContextType = useMemo(
    () => ({
      setShowBlockWhenDisconnectedAlert,
      showBlockWhenDisconnectedAlert,
    }),
    [setShowBlockWhenDisconnectedAlert, showBlockWhenDisconnectedAlert],
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
  const blockWhenDisconnected = useSelector((state) => state.settings.blockWhenDisconnected);

  let recoveryAction: RecoveryAction;

  if (blockWhenDisconnected && isBlocked) {
    recoveryAction = RecoveryAction.disableBlockedWhenDisconnected;
  } else if (!blockWhenDisconnected && isBlocked) {
    recoveryAction = RecoveryAction.disconnect;
  } else {
    recoveryAction = RecoveryAction.openBrowser;
  }

  let recoveryMessage: string;

  switch (recoveryAction) {
    case RecoveryAction.openBrowser:
    case RecoveryAction.disableBlockedWhenDisconnected:
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
