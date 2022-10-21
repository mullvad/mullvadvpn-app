import React, { useCallback } from 'react';
import { sprintf } from 'sprintf-js';

import { colors } from '../../config.json';
import { AccountToken } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import { formatAccountToken } from '../lib/account';
import { formatHtml } from '../lib/html-formatter';
import { LoginState } from '../redux/account/reducers';
import { useSelector } from '../redux/store';
import Accordion from './Accordion';
import * as AppButton from './AppButton';
import { AriaControlGroup, AriaControlled, AriaControls } from './AriaGroup';
import { Brand, HeaderBarSettingsButton } from './HeaderBar';
import ImageView from './ImageView';
import { Container, Header, Layout } from './Layout';
import {
  StyledAccountDropdownContainer,
  StyledAccountDropdownItem,
  StyledAccountDropdownItemButton,
  StyledAccountDropdownItemButtonLabel,
  StyledAccountDropdownRemoveButton,
  StyledAccountDropdownRemoveIcon,
  StyledAccountInputBackdrop,
  StyledAccountInputGroup,
  StyledBlockMessage,
  StyledBlockMessageContainer,
  StyledBlockTitle,
  StyledDropdownSpacer,
  StyledFooter,
  StyledInput,
  StyledInputButton,
  StyledInputSubmitIcon,
  StyledLoginFooterPrompt,
  StyledLoginForm,
  StyledStatusIcon,
  StyledSubtitle,
  StyledTitle,
  StyledTopInfo,
} from './LoginStyles';

interface IProps {
  accountToken?: AccountToken;
  accountHistory?: AccountToken;
  loginState: LoginState;
  showBlockMessage: boolean;
  openExternalLink: (type: string) => void;
  login: (accountToken: AccountToken) => void;
  resetLoginError: () => void;
  updateAccountToken: (accountToken: AccountToken) => void;
  clearAccountHistory: () => Promise<void>;
  createNewAccount: () => void;
  isPerformingPostUpgrade?: boolean;
}

interface IState {
  isActive: boolean;
}

const MIN_ACCOUNT_TOKEN_LENGTH = 10;

export default class Login extends React.Component<IProps, IState> {
  public state: IState = {
    isActive: true,
  };

  private accountInput = React.createRef<HTMLInputElement>();
  private shouldResetLoginError = false;

  constructor(props: IProps) {
    super(props);

    if (props.loginState.type === 'failed') {
      this.shouldResetLoginError = true;
    }
  }

  public componentDidUpdate(prevProps: IProps, _prevState: IState) {
    if (
      this.props.loginState.type !== prevProps.loginState.type &&
      this.props.loginState.type === 'failed'
    ) {
      this.shouldResetLoginError = true;

      // focus on login field when failed to log in
      this.accountInput.current?.focus();
    }
  }

  public render() {
    const allowInteraction = this.allowInteraction();

    return (
      <Layout>
        <Header>
          <Brand />
          <HeaderBarSettingsButton disabled={!allowInteraction} />
        </Header>
        <Container>
          <StyledTopInfo>
            {this.props.showBlockMessage ? <BlockMessage /> : this.getStatusIcon()}
          </StyledTopInfo>

          <StyledLoginForm>
            <StyledTitle aria-live="polite">{this.formTitle()}</StyledTitle>

            {this.createLoginForm()}
          </StyledLoginForm>

          <StyledFooter show={allowInteraction}>{this.createFooter()}</StyledFooter>
        </Container>
      </Layout>
    );
  }

  private onFocus = () => {
    this.setState({ isActive: true });
  };

  private onBlur = (e: React.FocusEvent<HTMLInputElement>) => {
    // restore focus if click happened within dropdown
    if (e.relatedTarget) {
      if (this.accountInput.current) {
        this.accountInput.current.focus();
      }
      return;
    }

    this.setState({ isActive: false });
  };

  private onSubmit = (event?: React.FormEvent) => {
    event?.preventDefault();

    if (this.accountTokenValid()) {
      this.props.login(this.props.accountToken!);
    }
  };

  private onInputChange = (accountToken: string) => {
    // reset error when user types in the new account number
    if (this.shouldResetLoginError) {
      this.shouldResetLoginError = false;
      this.props.resetLoginError();
    }

    this.props.updateAccountToken(accountToken);
  };

  private formTitle() {
    if (this.props.isPerformingPostUpgrade) {
      return messages.pgettext('login-view', 'Upgrading...');
    }

    switch (this.props.loginState.type) {
      case 'logging in':
      case 'too many devices':
        return this.props.loginState.method === 'existing_account'
          ? messages.pgettext('login-view', 'Logging in...')
          : messages.pgettext('login-view', 'Creating account...');
      case 'failed':
        return this.props.loginState.method === 'existing_account'
          ? messages.pgettext('login-view', 'Login failed')
          : messages.pgettext('login-view', 'Error');
      case 'ok':
        return this.props.loginState.method === 'existing_account'
          ? messages.pgettext('login-view', 'Logged in')
          : messages.pgettext('login-view', 'Account created');
      default:
        return messages.pgettext('login-view', 'Login');
    }
  }

  private formSubtitle() {
    if (this.props.isPerformingPostUpgrade) {
      return messages.pgettext('login-view', 'Finishing upgrade.');
    }

    switch (this.props.loginState.type) {
      case 'failed':
        return this.props.loginState.method === 'existing_account'
          ? this.props.loginState.error.message || messages.pgettext('login-view', 'Unknown error')
          : messages.pgettext('login-view', 'Failed to create account');
      case 'too many devices':
        return messages.pgettext('login-view', 'Too many devices');
      case 'logging in':
        return this.props.loginState.method === 'existing_account'
          ? messages.pgettext('login-view', 'Checking account number')
          : messages.pgettext('login-view', 'Please wait');
      case 'ok':
        return this.props.loginState.method === 'existing_account'
          ? messages.pgettext('login-view', 'Valid account number')
          : messages.pgettext('login-view', 'Logged in');
      default:
        return messages.pgettext('login-view', 'Enter your account number');
    }
  }

  private getStatusIcon() {
    const statusIconPath = this.getStatusIconPath();
    return (
      <StyledStatusIcon>
        {statusIconPath ? <ImageView source={statusIconPath} height={48} width={48} /> : null}
      </StyledStatusIcon>
    );
  }

  private getStatusIconPath(): string | undefined {
    if (this.props.isPerformingPostUpgrade) {
      return 'icon-spinner';
    }

    switch (this.props.loginState.type) {
      case 'logging in':
        return 'icon-spinner';
      case 'failed':
        return 'icon-fail';
      case 'ok':
        return 'icon-success';
      default:
        return undefined;
    }
  }

  private allowInteraction() {
    return (
      !this.props.isPerformingPostUpgrade &&
      this.props.loginState.type !== 'logging in' &&
      this.props.loginState.type !== 'ok' &&
      this.props.loginState.type !== 'too many devices'
    );
  }

  private allowCreateAccount() {
    const { accountToken } = this.props;
    return this.allowInteraction() && (accountToken === undefined || accountToken.length === 0);
  }

  private accountTokenValid(): boolean {
    const { accountToken } = this.props;
    return accountToken !== undefined && accountToken.length >= MIN_ACCOUNT_TOKEN_LENGTH;
  }

  private shouldShowAccountHistory() {
    return this.allowInteraction() && this.props.accountHistory !== undefined;
  }

  private onSelectAccountFromHistory = (accountToken: string) => {
    this.props.updateAccountToken(accountToken);
    this.props.login(accountToken);
  };

  private onClearAccountHistory = () => {
    void this.clearAccountHistory();
  };

  private async clearAccountHistory() {
    try {
      await this.props.clearAccountHistory();

      // TODO: Remove account from memory
    } catch (error) {
      // TODO: Show error
    }
  }

  private createLoginForm() {
    const allowInteraction = this.allowInteraction();
    const allowLogin = allowInteraction && this.accountTokenValid();
    const hasError =
      this.props.loginState.type === 'failed' &&
      this.props.loginState.method === 'existing_account';

    return (
      <>
        <StyledSubtitle>{this.formSubtitle()}</StyledSubtitle>
        <StyledAccountInputGroup
          active={allowInteraction && this.state.isActive}
          editable={allowInteraction}
          error={hasError}
          onSubmit={this.onSubmit}>
          <StyledAccountInputBackdrop>
            <StyledInput
              allowedCharacters="[0-9]"
              separator=" "
              groupLength={4}
              placeholder="0000 0000 0000 0000"
              value={this.props.accountToken || ''}
              disabled={!allowInteraction}
              onFocus={this.onFocus}
              onBlur={this.onBlur}
              handleChange={this.onInputChange}
              autoFocus={true}
              ref={this.accountInput}
              aria-autocomplete="list"
            />
            <StyledInputButton
              type="submit"
              visible={allowLogin}
              disabled={!allowLogin}
              aria-label={
                // TRANSLATORS: This is used by screenreaders to communicate the login button.
                messages.pgettext('accessibility', 'Login')
              }>
              <StyledInputSubmitIcon
                visible={
                  this.props.loginState.type !== 'logging in' && !this.props.isPerformingPostUpgrade
                }
                source="icon-arrow"
                height={16}
                width={24}
                tintColor="rgb(255, 255, 255)"
              />
            </StyledInputButton>
          </StyledAccountInputBackdrop>
          <Accordion expanded={this.shouldShowAccountHistory()}>
            <StyledAccountDropdownContainer>
              <AccountDropdown
                item={this.props.accountHistory}
                onSelect={this.onSelectAccountFromHistory}
                onRemove={this.onClearAccountHistory}
              />
            </StyledAccountDropdownContainer>
          </Accordion>
        </StyledAccountInputGroup>
      </>
    );
  }

  private createFooter() {
    return (
      <>
        <StyledLoginFooterPrompt>
          {messages.pgettext('login-view', 'Don’t have an account number?')}
        </StyledLoginFooterPrompt>
        <AppButton.BlueButton
          onClick={this.props.createNewAccount}
          disabled={!this.allowCreateAccount()}>
          {messages.pgettext('login-view', 'Create account')}
        </AppButton.BlueButton>
      </>
    );
  }
}

interface IAccountDropdownProps {
  item?: AccountToken;
  onSelect: (value: AccountToken) => void;
  onRemove: (value: AccountToken) => void;
}

function AccountDropdown(props: IAccountDropdownProps) {
  const token = props.item;
  if (!token) {
    return null;
  }
  const label = formatAccountToken(token);
  return (
    <AccountDropdownItem
      value={token}
      label={label}
      onSelect={props.onSelect}
      onRemove={props.onRemove}
    />
  );
}

interface IAccountDropdownItemProps {
  label: string;
  value: AccountToken;
  onRemove: (value: AccountToken) => void;
  onSelect: (value: AccountToken) => void;
}

function AccountDropdownItem(props: IAccountDropdownItemProps) {
  const handleSelect = useCallback(() => {
    props.onSelect(props.value);
  }, [props.onSelect, props.value]);

  const handleRemove = useCallback(
    (event: React.MouseEvent<HTMLButtonElement>) => {
      // Prevent login form from submitting
      event.preventDefault();
      props.onRemove(props.value);
    },
    [props.onRemove, props.value],
  );

  return (
    <>
      <StyledDropdownSpacer />
      <StyledAccountDropdownItem>
        <AriaControlGroup>
          <AriaControlled>
            <StyledAccountDropdownItemButton id={props.label} onClick={handleSelect} type="button">
              <StyledAccountDropdownItemButtonLabel>
                {props.label}
              </StyledAccountDropdownItemButtonLabel>
            </StyledAccountDropdownItemButton>
          </AriaControlled>
          <AriaControls>
            <StyledAccountDropdownRemoveButton
              onClick={handleRemove}
              aria-controls={props.label}
              aria-label={
                // TRANSLATORS: This is used by screenreaders to communicate the "x" button next to a saved account number.
                // TRANSLATORS: Available placeholders:
                // TRANSLATORS: %(accountToken)s - the account token to the left of the button
                sprintf(messages.pgettext('accessibility', 'Forget %(accountToken)s'), {
                  accountToken: props.label,
                })
              }>
              <StyledAccountDropdownRemoveIcon
                tintColor={colors.blue40}
                tintHoverColor={colors.blue}
                source="icon-close-sml"
                height={16}
                width={16}
              />
            </StyledAccountDropdownRemoveButton>
          </AriaControls>
        </AriaControlGroup>
      </StyledAccountDropdownItem>
    </>
  );
}

function BlockMessage() {
  const { setBlockWhenDisconnected, disconnectTunnel } = useAppContext();
  const tunnelState = useSelector((state) => state.connection.status);
  const blockWhenDisconnected = useSelector((state) => state.settings.blockWhenDisconnected);

  const unlock = useCallback(() => {
    if (blockWhenDisconnected) {
      void setBlockWhenDisconnected(false);
    }

    if (tunnelState.state === 'error') {
      void disconnectTunnel();
    }
  }, [blockWhenDisconnected, tunnelState, setBlockWhenDisconnected, disconnectTunnel]);

  const lockdownModeSettingName = messages.pgettext('vpn-settings-view', 'Lockdown mode');
  const message = formatHtml(
    blockWhenDisconnected
      ? sprintf(
          // TRANSLATORS: This is a warning message shown when the app is blocking the users
          // TRANSLATORS: internet connection while logged out.
          // TRANSLATORS: Available placeholder:
          // TRANSLATORS: %(lockdownModeSettingName)s - The translation of "Lockdown mode"
          messages.pgettext(
            'login-view',
            '<b>%(lockdownModeSettingName)s</b> is enabled. Disable it to unblock your connection.',
          ),
          { lockdownModeSettingName },
        )
      : // This makes the translator comment appear on it's own line.
        // TRANSLATORS: This is a warning message shown when the app is blocking the users
        // TRANSLATORS: internet connection while logged out.
        messages.pgettext('login-view', 'Our kill switch is currently blocking your connection.'),
  );
  const buttonText = blockWhenDisconnected
    ? messages.gettext('Disable')
    : messages.gettext('Unblock');

  return (
    <StyledBlockMessageContainer>
      <StyledBlockTitle>{messages.gettext('Blocking internet')}</StyledBlockTitle>
      <StyledBlockMessage>{message}</StyledBlockMessage>
      <AppButton.RedButton onClick={unlock}>{buttonText}</AppButton.RedButton>
    </StyledBlockMessageContainer>
  );
}
