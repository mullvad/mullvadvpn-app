import React, { useCallback, useContext, useState } from 'react';
import { sprintf } from 'sprintf-js';

import { formatDate } from '../../shared/account-expiry';
import { VoucherResponse } from '../../shared/daemon-rpc-types';
import { formatRelativeDate } from '../../shared/date-helper';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import useActions from '../lib/actionsHook';
import accountActions from '../redux/account/actions';
import { useSelector } from '../redux/store';
import * as AppButton from './AppButton';
import ImageView from './ImageView';
import { ModalAlert } from './Modal';
import {
  StyledEmptyResponse,
  StyledErrorResponse,
  StyledInput,
  StyledLabel,
  StyledProgressResponse,
  StyledProgressWrapper,
  StyledSpinner,
  StyledStatusIcon,
  StyledTitle,
} from './RedeemVoucherStyles';

const MIN_VOUCHER_LENGTH = 16;

interface IRedeemVoucherContextValue {
  onSubmit: () => void;
  value: string;
  setValue: (value: string) => void;
  valueValid: boolean;
  submitting: boolean;
  response?: VoucherResponse;
}

const contextProviderMissingError = new Error('<RedeemVoucherContext.Provider> is missing');

const RedeemVoucherContext = React.createContext<IRedeemVoucherContextValue>({
  onSubmit() {
    throw contextProviderMissingError;
  },
  get value(): string {
    throw contextProviderMissingError;
  },
  setValue(_) {
    throw contextProviderMissingError;
  },
  get valueValid(): boolean {
    throw contextProviderMissingError;
  },
  get submitting(): boolean {
    throw contextProviderMissingError;
  },
  get response(): VoucherResponse {
    throw contextProviderMissingError;
  },
});

interface IRedeemVoucherProps {
  onSubmit?: () => void;
  onSuccess?: (newExpiry: string, secondsAdded: number) => void;
  onFailure?: () => void;
  children?: React.ReactNode;
}

export function RedeemVoucherContainer(props: IRedeemVoucherProps) {
  const { onSubmit, onSuccess, onFailure } = props;

  const { submitVoucher } = useAppContext();
  const { updateAccountExpiry } = useActions(accountActions);

  const [value, setValue] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [response, setResponse] = useState<VoucherResponse>();

  const valueValid = value.length >= MIN_VOUCHER_LENGTH;

  const onSubmitWrapper = useCallback(async () => {
    if (!valueValid) {
      return;
    }

    const submitTimestamp = Date.now();
    setSubmitting(true);
    onSubmit?.();
    const response = await submitVoucher(value);

    // Show the spinner for at least half a second if it isn't successful.
    const submitDuration = Date.now() - submitTimestamp;
    if (response.type !== 'success' && submitDuration < 500) {
      await new Promise((resolve) => setTimeout(resolve, 500 - submitDuration));
    }

    setSubmitting(false);
    setResponse(response);
    if (response.type === 'success') {
      onSuccess?.(response.newExpiry, response.secondsAdded);
    } else {
      onFailure?.();
    }
  }, [value, valueValid, onSubmit, submitVoucher, updateAccountExpiry, onSuccess, onFailure]);

  return (
    <RedeemVoucherContext.Provider
      value={{ onSubmit: onSubmitWrapper, value, setValue, valueValid, submitting, response }}>
      {props.children}
    </RedeemVoucherContext.Provider>
  );
}

interface IRedeemVoucherInputProps {
  className?: string;
}

export function RedeemVoucherInput(props: IRedeemVoucherInputProps) {
  const { value, setValue, onSubmit, submitting, response } = useContext(RedeemVoucherContext);
  const disabled = submitting || response?.type === 'success';

  const handleChange = useCallback(
    (value: string) => {
      setValue(value);
    },
    [setValue],
  );

  const onKeyPress = useCallback(
    (event: React.KeyboardEvent<HTMLInputElement>) => {
      if (event.key === 'Enter') {
        onSubmit();
      }
    },
    [onSubmit],
  );

  return (
    <StyledInput
      className={props.className}
      allowedCharacters="[A-Z0-9]"
      separator="-"
      uppercaseOnly
      groupLength={4}
      maxLength={16}
      addTrailingSeparator
      disabled={disabled}
      value={value}
      placeholder={'XXXX-XXXX-XXXX-XXXX'}
      handleChange={handleChange}
      onKeyPress={onKeyPress}
    />
  );
}

export function RedeemVoucherResponse() {
  const { response, submitting } = useContext(RedeemVoucherContext);

  if (submitting) {
    return (
      <>
        <StyledProgressWrapper>
          <StyledSpinner source="icon-spinner" height={20} width={20} />
          <StyledProgressResponse>
            {messages.pgettext('redeem-voucher-view', 'Verifying voucher...')}
          </StyledProgressResponse>
        </StyledProgressWrapper>
      </>
    );
  }

  if (response) {
    switch (response.type) {
      case 'success':
        return <StyledEmptyResponse />;
      case 'invalid':
        return (
          <StyledErrorResponse>
            {messages.pgettext('redeem-voucher-view', 'Voucher code is invalid.')}
          </StyledErrorResponse>
        );
      case 'already_used':
        return (
          <StyledErrorResponse>
            {messages.pgettext('redeem-voucher-view', 'Voucher code has already been used.')}
          </StyledErrorResponse>
        );
      case 'error':
        return (
          <StyledErrorResponse>
            {messages.pgettext('redeem-voucher-view', 'An error occurred.')}
          </StyledErrorResponse>
        );
    }
  }

  return <StyledEmptyResponse />;
}

export function RedeemVoucherSubmitButton() {
  const { valueValid, onSubmit, submitting, response } = useContext(RedeemVoucherContext);
  const disabled = submitting || response?.type === 'success';

  return (
    <AppButton.GreenButton disabled={!valueValid || disabled} onClick={onSubmit}>
      {messages.pgettext('redeem-voucher-view', 'Redeem')}
    </AppButton.GreenButton>
  );
}

interface IRedeemVoucherAlertProps {
  show: boolean;
  onClose?: () => void;
}

export function RedeemVoucherAlert(props: IRedeemVoucherAlertProps) {
  const { submitting, response } = useContext(RedeemVoucherContext);
  const locale = useSelector((state) => state.userInterface.locale);

  if (response?.type === 'success') {
    const duration = formatRelativeDate(response.secondsAdded * 1000, 0);
    const expiry = formatDate(response.newExpiry, locale);

    return (
      <ModalAlert
        isOpen={props.show}
        buttons={[
          <AppButton.BlueButton key="gotit" onClick={props.onClose}>
            {messages.gettext('Got it!')}
          </AppButton.BlueButton>,
        ]}
        close={props.onClose}>
        <StyledStatusIcon>
          <ImageView source="icon-success" height={60} width={60} />
        </StyledStatusIcon>
        <StyledTitle>
          {messages.pgettext('redeem-voucher-view', 'Voucher was successfully redeemed.')}
        </StyledTitle>
        <StyledLabel>
          {sprintf(messages.gettext('%(duration)s was added, account paid until %(expiry)s.'), {
            duration,
            expiry,
          })}
        </StyledLabel>
      </ModalAlert>
    );
  } else {
    return (
      <ModalAlert
        isOpen={props.show}
        buttons={[
          <RedeemVoucherSubmitButton key="submit" />,
          <AppButton.BlueButton key="cancel" disabled={submitting} onClick={props.onClose}>
            {messages.pgettext('redeem-voucher-alert', 'Cancel')}
          </AppButton.BlueButton>,
        ]}
        close={props.onClose}>
        <StyledLabel>{messages.pgettext('redeem-voucher-alert', 'Enter voucher code')}</StyledLabel>
        <RedeemVoucherInput />
        <RedeemVoucherResponse />
      </ModalAlert>
    );
  }
}

interface IRedeemVoucherButtonProps {
  className?: string;
}

export function RedeemVoucherButton(props: IRedeemVoucherButtonProps) {
  const [showAlert, setShowAlert] = useState(false);

  const onClick = useCallback(() => setShowAlert(true), []);
  const onClose = useCallback(() => setShowAlert(false), []);

  return (
    <>
      <AppButton.GreenButton onClick={onClick} className={props.className}>
        {messages.pgettext('redeem-voucher-alert', 'Redeem voucher')}
      </AppButton.GreenButton>
      <RedeemVoucherContainer>
        <RedeemVoucherAlert show={showAlert} onClose={onClose} />
      </RedeemVoucherContainer>
    </>
  );
}
