import { useCallback } from 'react';
import { useParams } from 'react-router';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { colors, links } from '../../config.json';
import { formatDate } from '../../shared/account-expiry';
import { formatRelativeDate } from '../../shared/date-helper';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import useActions from '../lib/actionsHook';
import { transitions, useHistory } from '../lib/history';
import { generateRoutePath, RoutePath } from '../lib/routes';
import account from '../redux/account/actions';
import { useSelector } from '../redux/store';
import * as AppButton from './AppButton';
import { AriaDescribed, AriaDescription, AriaDescriptionGroup } from './AriaGroup';
import { hugeText, measurements, tinyText } from './common-styles';
import CustomScrollbars from './CustomScrollbars';
import { calculateHeaderBarStyle, DefaultHeaderBar, HeaderBarStyle } from './HeaderBar';
import ImageView from './ImageView';
import { Container, Footer, Layout } from './Layout';
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
  padding: `0 ${measurements.viewMargin}`,
  paddingBottom: 'auto',
});

export const StyledTitle = styled.span(hugeText, {
  lineHeight: '38px',
  marginBottom: '8px',
});

export const StyledLabel = styled.span(tinyText, {
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

  const onSuccess = useCallback(
    (newExpiry: string, secondsAdded: number) => {
      const path = generateRoutePath(RoutePath.voucherSuccess, { newExpiry, secondsAdded });
      history.push(path);
    },
    [history],
  );

  const navigateBack = useCallback(() => {
    history.pop();
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
              <RedeemVoucherResponse />
            </StyledBody>

            <Footer>
              <AppButton.ButtonGroup>
                <RedeemVoucherSubmitButton />
                <AppButton.BlueButton onClick={navigateBack}>
                  {messages.gettext('Cancel')}
                </AppButton.BlueButton>
              </AppButton.ButtonGroup>
            </Footer>
          </RedeemVoucherContainer>
        </StyledContainer>
      </StyledCustomScrollbars>
    </Layout>
  );
}

export function VoucherVerificationSuccess() {
  const { newExpiry, secondsAdded } = useParams<{ newExpiry: string; secondsAdded: string }>();

  return (
    <TimeAdded
      newExpiry={newExpiry}
      secondsAdded={parseInt(secondsAdded)}
      title={messages.pgettext('connect-view', 'Voucher was successfully redeemed')}
    />
  );
}

interface ITimeAddedProps {
  title?: string;
  newExpiry?: string;
  secondsAdded?: number;
}

export function TimeAdded(props: ITimeAddedProps) {
  const history = useHistory();
  const finish = useFinishedCallback();
  const expiry = useSelector((state) => state.account.expiry);
  const isNewAccount = useSelector(
    (state) => state.account.status.type === 'ok' && state.account.status.method === 'new_account',
  );
  const locale = useSelector((state) => state.userInterface.locale);

  const navigateToSetupFinished = useCallback(() => {
    if (isNewAccount) {
      history.push(RoutePath.setupFinished);
    } else {
      finish();
    }
  }, [history, finish]);

  const duration =
    props.secondsAdded !== undefined ? formatRelativeDate(props.secondsAdded * 1000, 0) : undefined;

  let newExpiry = '';
  if (props.newExpiry !== undefined) {
    newExpiry = formatDate(props.newExpiry, locale);
  } else if (expiry !== undefined) {
    newExpiry = formatDate(expiry, locale);
  }

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
              {duration
                ? sprintf(
                    messages.gettext('%(duration)s was added, account paid until %(expiry)s.'),
                    {
                      duration,
                      expiry: newExpiry,
                    },
                  )
                : sprintf(messages.gettext('Account paid until %(expiry)s.'), {
                    expiry: newExpiry,
                  })}
            </StyledLabel>
          </StyledBody>

          <Footer>
            <AppButton.BlueButton onClick={navigateToSetupFinished}>
              {messages.gettext('Next')}
            </AppButton.BlueButton>
          </Footer>
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
            <StyledTitle>{messages.pgettext('connect-view', 'You’re all set!')}</StyledTitle>
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

          <Footer>
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
          </Footer>
        </StyledContainer>
      </StyledCustomScrollbars>
    </Layout>
  );
}

function HeaderBar() {
  const isNewAccount = useSelector(
    (state) => state.account.status.type === 'ok' && state.account.status.method === 'new_account',
  );
  const tunnelState = useSelector((state) => state.connection.status);
  const headerBarStyle = isNewAccount
    ? HeaderBarStyle.default
    : calculateHeaderBarStyle(tunnelState);

  return <StyledHeader barStyle={headerBarStyle} />;
}

function useFinishedCallback() {
  const { accountSetupFinished } = useActions(account);

  const history = useHistory();
  const isNewAccount = useSelector(
    (state) => state.account.status.type === 'ok' && state.account.status.method === 'new_account',
  );

  const callback = useCallback(() => {
    // Changes login method from "new_account" to "existing_account"
    if (isNewAccount) {
      accountSetupFinished();
    }

    history.reset(RoutePath.main, { transition: transitions.push });
  }, [isNewAccount, accountSetupFinished, history]);

  return callback;
}
