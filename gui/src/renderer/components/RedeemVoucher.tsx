import * as React from 'react';
import { Component, Text, TextInput, View } from 'reactxp';
import { colors } from '../../config.json';
import { VoucherResponse } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import * as AppButton from './AppButton';
import styles from './RedeemVoucherStyles';

interface IRedeemVoucherContextValue {
  onSubmit: () => void;
  value: string;
  setValue: (value: string) => void;
  valueValid: boolean;
  waiting: boolean;
  response?: VoucherResponse;
}

const RedeemVoucherContext = React.createContext<IRedeemVoucherContextValue>({
  onSubmit() {
    throw new Error('<RedeemVoucherContext.Provider> is missing');
  },
  get value(): string {
    throw new Error('<redeemvouchercontext.provider> is missing');
  },
  setValue(_) {
    throw new Error('<RedeemVoucherContext.Provider> is missing');
  },
  get valueValid(): boolean {
    throw new Error('<redeemvouchercontext.provider> is missing');
  },
  get waiting(): boolean {
    throw new Error('<redeemvouchercontext.provider> is missing');
  },
  get response(): VoucherResponse {
    throw new Error('<redeemvouchercontext.provider> is missing');
  },
});

interface IRedeemVoucherProps {
  submitVoucher: (voucherCode: string) => Promise<VoucherResponse>;
  updateAccountExpiry: (expiry: string) => void;
  onSubmit?: () => void;
  onSuccess?: () => void;
  onFailure?: () => void;
  children?: React.ReactNode;
}

interface IRedeemVoucherState {
  value: string;
  waiting: boolean;
  response?: VoucherResponse;
}

export class RedeemVoucher extends Component<IRedeemVoucherProps, IRedeemVoucherState> {
  public state = {
    value: '',
    waiting: false,
    response: undefined,
  };

  public render() {
    return (
      <RedeemVoucherContext.Provider
        value={{
          onSubmit: this.onSubmit,
          value: this.state.value,
          setValue: this.setValue,
          valueValid: RedeemVoucher.isValueValid(this.state.value),
          waiting: this.state.waiting,
          response: this.state.response,
        }}>
        {this.props.children}
      </RedeemVoucherContext.Provider>
    );
  }

  private setValue = (value: string) => {
    this.setState({ value });
  };

  private static isValueValid(value: string): boolean {
    return value.length >= 16;
  }

  private onSubmit = async () => {
    if (!RedeemVoucher.isValueValid(this.state.value)) {
      return;
    }

    this.setState({ waiting: true });

    if (this.props.onSubmit) {
      this.props.onSubmit();
    }

    const response = await this.props.submitVoucher(this.state.value);

    if (response.type === 'success') {
      this.setState({ value: '', waiting: false, response });
      this.props.updateAccountExpiry(response.new_expiry);
      if (this.props.onSuccess) {
        this.props.onSuccess();
      }
    } else {
      this.setState({ waiting: false, response });
      if (this.props.onFailure) {
        this.props.onFailure();
      }
    }
  };
}

export class RedeemVoucherInput extends Component {
  public render() {
    return (
      <RedeemVoucherContext.Consumer>
        {(context) => (
          <View>
            <TextInput
              style={styles.textInput}
              value={context.value}
              placeholder={'XXXX-XXXX-XXXX-XXXX'}
              placeholderTextColor={colors.blue40}
              autoCorrect={false}
              onChangeText={context.setValue}
              onSubmitEditing={context.onSubmit}
            />
          </View>
        )}
      </RedeemVoucherContext.Consumer>
    );
  }
}

export class RedeemVoucherResponse extends Component {
  public render() {
    return (
      <RedeemVoucherContext.Consumer>
        {(context) => {
          if (context.response) {
            switch (context.response.type) {
              case 'success':
                return (
                  <Text style={styles.redeemVoucherResponseSuccess}>
                    {messages.pgettext('redeem-voucher-view', 'Voucher was successfully redeemed.')}
                  </Text>
                );
              case 'invalid':
                return (
                  <Text style={styles.redeemVoucherResponseError}>
                    {messages.pgettext('redeem-voucher-view', 'Voucher code is invalid.')}
                  </Text>
                );
              case 'already_used':
                return (
                  <Text style={styles.redeemVoucherResponseError}>
                    {messages.pgettext(
                      'redeem-voucher-view',
                      'Voucher code has already been used.',
                    )}
                  </Text>
                );
              case 'error':
                return (
                  <Text style={styles.redeemVoucherResponseError}>
                    {messages.pgettext('redeem-voucher-view', 'An error occured.')}
                  </Text>
                );
            }
          }

          return <View style={styles.redeemVoucherResponseEmpty} />;
        }}
      </RedeemVoucherContext.Consumer>
    );
  }
}

export class RedeemVoucherSubmitButton extends Component {
  public render() {
    return (
      <RedeemVoucherContext.Consumer>
        {(context) => (
          <AppButton.GreenButton
            key="cancel"
            disabled={!context.valueValid || context.waiting}
            onPress={context.onSubmit}>
            {messages.pgettext('redeem-voucher-view', 'Redeem')}
          </AppButton.GreenButton>
        )}
      </RedeemVoucherContext.Consumer>
    );
  }
}
