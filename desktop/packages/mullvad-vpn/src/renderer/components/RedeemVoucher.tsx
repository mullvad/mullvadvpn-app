import React, { useCallback, useContext, useState } from 'react';
import { sprintf } from 'sprintf-js';

import { formatDate } from '../../shared/account-expiry';
import { VoucherResponse } from '../../shared/daemon-rpc-types';
import { formatRelativeDate } from '../../shared/date-helper';
import { messages } from '../../shared/gettext';
import { isAccountNumber } from '../../shared/utils';
import { useAppContext } from '../context';
import { Button, ButtonProps, Flex, LabelTinySemiBold, Spinner } from '../lib/components';
import { Dialog } from '../lib/components/dialog';
import { FlexColumn } from '../lib/components/flex-column';
import { useSelector } from '../redux/store';
import { StyledEmptyResponse, StyledInput } from './RedeemVoucherStyles';
import { StatusDialog } from './status-dialog';

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
      <Flex alignItems="center" gap="small">
        <Spinner size="small" />
        <LabelTinySemiBold>
          {messages.pgettext('redeem-voucher-view', 'Verifying voucher...')}
        </LabelTinySemiBold>
      </Flex>
    );
  }

  if (response) {
    switch (response.type) {
      case 'success':
        return <StyledEmptyResponse />;
      case 'invalid':
        return (
          <FlexColumn gap="medium">
            <LabelTinySemiBold color="red">
              {messages.pgettext('redeem-voucher-view', 'Voucher code is invalid.')}
            </LabelTinySemiBold>
            {isAccountNumber(submittedValue) ? (
              <LabelTinySemiBold>
                {messages.pgettext(
                  'redeem-voucher-view',
                  'It looks like you’ve entered an account number instead of a voucher code. If you would like to change the active account, please log out first.',
                )}
              </LabelTinySemiBold>
            ) : null}
          </FlexColumn>
        );
      case 'already_used':
        return (
          <LabelTinySemiBold color="red">
            {messages.pgettext('redeem-voucher-view', 'Voucher code has already been used.')}
          </LabelTinySemiBold>
        );
      case 'error':
        return (
          <LabelTinySemiBold color="red">
            {messages.pgettext('redeem-voucher-view', 'An error occurred.')}
          </LabelTinySemiBold>
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

type RedeemVoucherAlertProps = {
  open: boolean;
  onOpenChange?: (open: boolean) => void;
};

export function RedeemVoucherAlert({
  open,
  onOpenChange: onOpenChangeProps,
}: RedeemVoucherAlertProps) {
  const { submitting, response } = useContext(RedeemVoucherContext);
  const locale = useSelector((state) => state.userInterface.locale);

  const onOpenChange = useCallback(
    (open: boolean) => {
      if (!submitting) {
        onOpenChangeProps?.(open);
      }
    },
    [submitting, onOpenChangeProps],
  );

  if (response?.type === 'success') {
    const duration = formatRelativeDate(0, response.secondsAdded * 1000, {
      capitalize: true,
      displayMonths: true,
    });
    const expiry = formatDate(response.newExpiry, locale);

    return (
      <StatusDialog variant="success" open={open} onOpenChange={onOpenChange}>
        <StatusDialog.Subtitle>
          {messages.pgettext('redeem-voucher-view', 'Voucher was successfully redeemed.')}
        </StatusDialog.Subtitle>
        <StatusDialog.Text>
          {sprintf(messages.gettext('%(duration)s was added, account paid until %(expiry)s.'), {
            duration,
            expiry,
          })}
        </StatusDialog.Text>
        <StatusDialog.CloseButton>
          <StatusDialog.CloseButton.Text>
            {messages.gettext('Got it!')}
          </StatusDialog.CloseButton.Text>
        </StatusDialog.CloseButton>
      </StatusDialog>
    );
  } else {
    return (
      <Dialog open={open} onOpenChange={onOpenChange}>
        <Dialog.Portal>
          <Dialog.Popup>
            <Dialog.PopupContent>
              <FlexColumn gap="tiny">
                <LabelTinySemiBold>
                  {
                    // TRANSLATORS: Input field label for voucher code.
                    messages.pgettext('redeem-voucher-alert', 'Enter voucher code')
                  }
                </LabelTinySemiBold>
                <FlexColumn gap="small">
                  <RedeemVoucherInput />
                  <RedeemVoucherResponse />
                </FlexColumn>
              </FlexColumn>
              <Dialog.ButtonGroup>
                <RedeemVoucherSubmitButton />
                <Dialog.CloseButton disabled={submitting}>
                  <Dialog.CloseButton.Text>
                    {
                      // TRANSLATORS: Cancel button label for voucher redemption.
                      messages.pgettext('redeem-voucher-alert', 'Cancel')
                    }
                  </Dialog.CloseButton.Text>
                </Dialog.CloseButton>
              </Dialog.ButtonGroup>
            </Dialog.PopupContent>
          </Dialog.Popup>
        </Dialog.Portal>
      </Dialog>
    );
  }
}

type RedeemVoucherButtonProps = ButtonProps;

export function RedeemVoucherButton(props: RedeemVoucherButtonProps) {
  const [showAlert, setShowAlert] = useState(false);

  const onClick = useCallback(() => setShowAlert(true), []);

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
        <RedeemVoucherAlert open={showAlert} onOpenChange={setShowAlert} />
      </RedeemVoucherContainer>
    </>
  );
}
