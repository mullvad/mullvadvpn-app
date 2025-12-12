import React, { useCallback, useContext, useState } from 'react';
import { sprintf } from 'sprintf-js';

import { formatDate } from '../../shared/account-expiry';
import { VoucherResponse } from '../../shared/daemon-rpc-types';
import { formatRelativeDate } from '../../shared/date-helper';
import { messages } from '../../shared/gettext';
import { isAccountNumber } from '../../shared/utils';
import { useAppContext } from '../context';
import { Button, ButtonProps, Flex, Spinner } from '../lib/components';
import { IconBadge } from '../lib/icon-badge';
import { useSelector } from '../redux/store';
import { ModalAlert } from './Modal';
import {
  StyledAccountNumberInfo,
  StyledEmptyResponse,
  StyledErrorResponse,
  StyledInput,
  StyledLabel,
  StyledProgressResponse,
  StyledTitle,
} from './RedeemVoucherStyles';

const MIN_VOUCHER_LENGTH = 16;

interface IRedeemVoucherContextValue {
  onSubmit: () => void;
  value: string;
  setValue: (value: string) => void;
  valueValid: boolean;
  submittedValue: string;
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
  get submittedValue(): string {
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

  const [value, setValue] = useState('');
  const [submittedValue, setSubmittedValue] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [response, setResponse] = useState<VoucherResponse>();

  const valueValid = value.length >= MIN_VOUCHER_LENGTH;

  const onSubmitWrapper = useCallback(async () => {
    if (!valueValid) {
      return;
    }

    const submitTimestamp = Date.now();
    setSubmitting(true);
    setSubmittedValue(value);
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
  }, [value, valueValid, onSubmit, submitVoucher, onSuccess, onFailure]);

  return (
    <RedeemVoucherContext.Provider
      value={{
        onSubmit: onSubmitWrapper,
        value,
        setValue,
        valueValid,
        submittedValue,
        submitting,
        response,
      }}>
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
  const { response, submitting, submittedValue } = useContext(RedeemVoucherContext);

  if (submitting) {
    return (
      <Flex alignItems="center" margin={{ top: 'small' }} gap="small">
        <Spinner size="medium" />
        <StyledProgressResponse>
          {messages.pgettext('redeem-voucher-view', 'Verifying voucher...')}
        </StyledProgressResponse>
      </Flex>
    );
  }

  if (response) {
    switch (response.type) {
      case 'success':
        return <StyledEmptyResponse />;
      case 'invalid':
        return (
          <>
            <StyledErrorResponse>
              {messages.pgettext('redeem-voucher-view', 'Voucher code is invalid.')}
            </StyledErrorResponse>
            {isAccountNumber(submittedValue) ? (
              <StyledAccountNumberInfo>
                {messages.pgettext(
                  'redeem-voucher-view',
                  'It looks like you’ve entered an account number instead of a voucher code. If you would like to change the active account, please log out first.',
                )}
              </StyledAccountNumberInfo>
            ) : null}
          </>
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
    <Button variant="success" disabled={!valueValid || disabled} onClick={onSubmit}>
      <Button.Text>
        {
          // TRANSLATORS: Button label for voucher redemption.
          messages.pgettext('redeem-voucher-view', 'Redeem')
        }
      </Button.Text>
    </Button>
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
    const duration = formatRelativeDate(0, response.secondsAdded * 1000, {
      capitalize: true,
      displayMonths: true,
    });
    const expiry = formatDate(response.newExpiry, locale);

    return (
      <ModalAlert
        isOpen={props.show}
        buttons={[
          <Button key="gotit" onClick={props.onClose}>
            <Button.Text>{messages.gettext('Got it!')}</Button.Text>
          </Button>,
        ]}
        close={props.onClose}>
        <Flex justifyContent="center" margin={{ top: 'large', bottom: 'medium' }}>
          <IconBadge state="positive" />
        </Flex>
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
          <Button key="cancel" disabled={submitting} onClick={props.onClose}>
            <Button.Text>
              {
                // TRANSLATORS: Cancel button label for voucher redemption.
                messages.pgettext('redeem-voucher-alert', 'Cancel')
              }
            </Button.Text>
          </Button>,
        ]}
        close={props.onClose}>
        <StyledLabel>
          {
            // TRANSLATORS: Input field label for voucher code.
            messages.pgettext('redeem-voucher-alert', 'Enter voucher code')
          }
        </StyledLabel>
        <RedeemVoucherInput />
        <RedeemVoucherResponse />
      </ModalAlert>
    );
  }
}

type RedeemVoucherButtonProps = ButtonProps;

export function RedeemVoucherButton(props: RedeemVoucherButtonProps) {
  const [showAlert, setShowAlert] = useState(false);

  const onClick = useCallback(() => setShowAlert(true), []);
  const onClose = useCallback(() => setShowAlert(false), []);

  return (
    <>
      <Button variant="success" onClick={onClick} {...props}>
        <Button.Text>
          {
            // TRANSLATORS: Button label for redeeming a voucher.
            messages.pgettext('redeem-voucher-alert', 'Redeem voucher')
          }
        </Button.Text>
      </Button>
      <RedeemVoucherContainer>
        <RedeemVoucherAlert show={showAlert} onClose={onClose} />
      </RedeemVoucherContainer>
    </>
  );
}
