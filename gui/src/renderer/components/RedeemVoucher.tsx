import React, { useCallback, useContext, useState } from 'react';
import { useSelector } from 'react-redux';
import { VoucherResponse } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { useScheduler } from '../../shared/scheduler';
import { useAppContext } from '../context';
import useActions from '../lib/actionsHook';
import accountActions from '../redux/account/actions';
import { IReduxState } from '../redux/store';
import * as AppButton from './AppButton';
import { ModalAlert } from './Modal';
import {
  StyledEmptyResponse,
  StyledErrorResponse,
  StyledInput,
  StyledLabel,
  StyledSpinner,
  StyledSuccessResponse,
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
  onSuccess?: () => void;
  onFailure?: () => void;
  children?: React.ReactNode;
}

export function RedeemVoucherContainer(props: IRedeemVoucherProps) {
  const { onSubmit, onSuccess, onFailure } = props;

  const closeScheduler = useScheduler();
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

    setSubmitting(true);
    onSubmit?.();
    const response = await submitVoucher(value);

    setSubmitting(false);
    setResponse(response);
    if (response.type === 'success') {
      setValue('');
      closeScheduler.schedule(() => {
        updateAccountExpiry(response.newExpiry);
        onSuccess?.();
      }, 1000);
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

export function RedeemVoucherInput() {
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
    return <StyledSpinner source="icon-spinner" height={20} width={20} />;
  }

  if (response) {
    switch (response.type) {
      case 'success':
        return (
          <StyledSuccessResponse>
            {messages.pgettext('redeem-voucher-view', 'Voucher was successfully redeemed.')}
          </StyledSuccessResponse>
        );
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
    <AppButton.GreenButton key="cancel" disabled={!valueValid || disabled} onClick={onSubmit}>
      {messages.pgettext('redeem-voucher-view', 'Redeem')}
    </AppButton.GreenButton>
  );
}

interface IRedeemVoucherAlertProps {
  onClose?: () => void;
}

export function RedeemVoucherAlert(props: IRedeemVoucherAlertProps) {
  const { submitting, response } = useContext(RedeemVoucherContext);
  const cancelDisabled = submitting || response?.type === 'success';

  return (
    <ModalAlert
      buttons={[
        <RedeemVoucherSubmitButton key="submit" />,
        <AppButton.BlueButton key="cancel" disabled={cancelDisabled} onClick={props.onClose}>
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

interface IRedeemVoucherButtonProps {
  className?: string;
}

export function RedeemVoucherButton(props: IRedeemVoucherButtonProps) {
  const isBlocked = useSelector((state: IReduxState) => state.connection.isBlocked);
  const [showAlert, setShowAlert] = useState(false);

  const onClick = useCallback(() => setShowAlert(true), []);
  const onClose = useCallback(() => setShowAlert(false), []);

  return (
    <>
      <AppButton.GreenButton disabled={isBlocked} onClick={onClick} className={props.className}>
        {messages.pgettext('redeem-voucher-alert', 'Redeem voucher')}
      </AppButton.GreenButton>
      {showAlert && (
        <RedeemVoucherContainer onSuccess={onClose}>
          <RedeemVoucherAlert onClose={onClose} />
        </RedeemVoucherContainer>
      )}
    </>
  );
}
