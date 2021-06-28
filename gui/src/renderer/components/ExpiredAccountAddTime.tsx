import React, { useCallback } from 'react';
import { useSelector } from 'react-redux';
import { useHistory } from 'react-router';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';
import { links, colors } from '../../config.json';
import { formatRelativeDate } from '../../shared/date-helper';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import useActions from '../lib/actionsHook';
import History from '../lib/history';
import account from '../redux/account/actions';
import { IReduxState } from '../redux/store';
import * as AppButton from './AppButton';
import { AriaDescribed, AriaDescription, AriaDescriptionGroup } from './AriaGroup';
import { bigText } from './common-styles';
import CustomScrollbars from './CustomScrollbars';
import { calculateHeaderBarStyle, DefaultHeaderBar, HeaderBarStyle } from './HeaderBar';
import ImageView from './ImageView';
import { Container, Layout } from './Layout';
import {
  RedeemVoucherContainer,
  RedeemVoucherInput,
  RedeemVoucherResponse,
  RedeemVoucherSubmitButton,
} from './RedeemVoucher';

export const StyledHeader = styled(DefaultHeaderBar)({
  flex: 0,
});

export const StyledCustomScrollbars = styled(CustomScrollbars)({
  flex: 1,
});

export const StyledContainer = styled(Container)({
  paddingTop: '22px',
  minHeight: '100%',
  backgroundColor: colors.darkBlue,
});

export const StyledBody = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  padding: '0 22px',
  paddingBottom: 'auto',
});

export const StyledFooter = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 0,
  padding: '18px 22px 22px',
});

export const StyledTitle = styled.span(bigText, {
  lineHeight: '38px',
  marginBottom: '8px',
});

export const StyledLabel = styled.span({
  fontFamily: 'Open Sans',
  fontSize: '13px',
  fontWeight: 600,
  lineHeight: '20px',
  color: colors.white,
  marginBottom: '9px',
});

export const StyledRedeemVoucherInput = styled(RedeemVoucherInput)({
  flex: 0,
});

export const StyledStatusIcon = styled.div({
  alignSelf: 'center',
  width: '60px',
  height: '60px',
  marginBottom: '18px',
});

export function VoucherInput() {
  const history = useHistory();

  const onSuccess = useCallback(() => {
    history.push('/main/voucher/success');
  }, [history]);

  const navigateBack = useCallback(() => {
    history.goBack();
  }, [history]);

  return (
    <Layout>
      <HeaderBar />
      <StyledCustomScrollbars fillContainer>
        <StyledContainer>
          <RedeemVoucherContainer onSuccess={onSuccess}>
            <StyledBody>
              <StyledTitle>{messages.pgettext('connect-view', 'Redeem voucher')}</StyledTitle>
              <StyledLabel>{messages.pgettext('connect-view', 'Enter voucher code')}</StyledLabel>
              <StyledRedeemVoucherInput />
              <RedeemVoucherResponse disableSuccessMessage />
            </StyledBody>

            <StyledFooter>
              <AppButton.ButtonGroup>
                <RedeemVoucherSubmitButton />
                <AppButton.BlueButton onClick={navigateBack}>
                  {messages.gettext('Cancel')}
                </AppButton.BlueButton>
              </AppButton.ButtonGroup>
            </StyledFooter>
          </RedeemVoucherContainer>
        </StyledContainer>
      </StyledCustomScrollbars>
    </Layout>
  );
}

export function VoucherVerificationSuccess() {
  return (
    <TimeAdded title={messages.pgettext('connect-view', 'Voucher was successfully redeemed')} />
  );
}

interface ITimeAddedProps {
  title?: string;
}

export function TimeAdded(props: ITimeAddedProps) {
  const history = useHistory();
  const finish = useFinishedCallback();
  const accountData = useSelector((state: IReduxState) => state.account);
  const isNewAccount = useSelector(
    (state: IReduxState) =>
      state.account.status.type === 'ok' && state.account.status.method === 'new_account',
  );

  const navigateToSetupFinished = useCallback(() => {
    if (isNewAccount) {
      history.push('/main/setup-finished');
    } else {
      finish();
    }
  }, [history, finish]);

  const duration =
    (accountData.expiry &&
      accountData.previousExpiry &&
      formatRelativeDate(accountData.expiry, accountData.previousExpiry)) ??
    '';

  return (
    <Layout>
      <HeaderBar />
      <StyledCustomScrollbars fillContainer>
        <StyledContainer>
          <StyledBody>
            <StyledStatusIcon>
              <ImageView source="icon-success" height={60} width={60} />
            </StyledStatusIcon>
            <StyledTitle>
              {props.title ?? messages.pgettext('connect-view', 'Time was successfully added')}
            </StyledTitle>
            <StyledLabel>
              {sprintf(
                messages.pgettext('connect-view', '%(duration)s was added to your account.'),
                { duration },
              )}
            </StyledLabel>
          </StyledBody>

          <StyledFooter>
            <AppButton.BlueButton onClick={navigateToSetupFinished}>
              {messages.gettext('Next')}
            </AppButton.BlueButton>
          </StyledFooter>
        </StyledContainer>
      </StyledCustomScrollbars>
    </Layout>
  );
}

export function SetupFinished() {
  const finish = useFinishedCallback();
  const { openUrl } = useAppContext();

  const openPrivacyLink = useCallback(() => openUrl(links.privacyGuide), [openUrl]);

  return (
    <Layout>
      <HeaderBar />
      <StyledCustomScrollbars fillContainer>
        <StyledContainer>
          <StyledBody>
            <StyledTitle>{messages.pgettext('connect-view', "You're all set!")}</StyledTitle>
            <StyledLabel>
              {messages.pgettext(
                'connect-view',
                'Go ahead and start using the app to begin reclaiming your online privacy.',
              )}
            </StyledLabel>
            <StyledLabel>
              {messages.pgettext(
                'connect-view',
                'To continue your journey as a privacy ninja, visit our website to pick up other privacy-friendly habits and tools.',
              )}
            </StyledLabel>
          </StyledBody>

          <StyledFooter>
            <AppButton.ButtonGroup>
              <AriaDescriptionGroup>
                <AriaDescribed>
                  <AppButton.BlueButton onClick={openPrivacyLink}>
                    <AppButton.Label>
                      {messages.pgettext('connect-view', 'Learn about privacy')}
                    </AppButton.Label>
                    <AriaDescription>
                      <AppButton.Icon
                        height={16}
                        width={16}
                        source="icon-extLink"
                        aria-label={messages.pgettext('accessibility', 'Opens externally')}
                      />
                    </AriaDescription>
                  </AppButton.BlueButton>
                </AriaDescribed>
              </AriaDescriptionGroup>
              <AppButton.GreenButton onClick={finish}>
                {messages.pgettext('connect-view', 'Start using the app')}
              </AppButton.GreenButton>
            </AppButton.ButtonGroup>
          </StyledFooter>
        </StyledContainer>
      </StyledCustomScrollbars>
    </Layout>
  );
}

function HeaderBar() {
  const isNewAccount = useSelector(
    (state: IReduxState) =>
      state.account.status.type === 'ok' && state.account.status.method === 'new_account',
  );
  const tunnelState = useSelector((state: IReduxState) => state.connection.status);
  const headerBarStyle = isNewAccount
    ? HeaderBarStyle.default
    : calculateHeaderBarStyle(tunnelState);

  return <StyledHeader barStyle={headerBarStyle} />;
}

function useFinishedCallback() {
  const { loggedIn } = useActions(account);

  const history = useHistory() as History;
  const isNewAccount = useSelector(
    (state: IReduxState) =>
      state.account.status.type === 'ok' && state.account.status.method === 'new_account',
  );

  const callback = useCallback(() => {
    // Changes login method from "new_account" to "existing_account"
    if (isNewAccount) {
      loggedIn();
    }

    history.resetWith('/main');
  }, [isNewAccount, loggedIn, history]);

  return callback;
}
