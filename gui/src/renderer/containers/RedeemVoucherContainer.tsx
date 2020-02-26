import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { RedeemVoucher } from '../components/RedeemVoucher';
import withAppContext, { IAppContext } from '../context';
import { ReduxDispatch } from '../redux/store';
import accountActions from '../redux/account/actions';

const mapDispatchToProps = (dispatch: ReduxDispatch, props: IAppContext) => {
  const account = bindActionCreators(accountActions, dispatch);

  return {
    submitVoucher: (voucherCode: string) => {
      return props.app.submitVoucher(voucherCode);
    },
    updateAccountExpiry: account.updateAccountExpiry,
  };
};

export default withAppContext(connect(null, mapDispatchToProps)(RedeemVoucher));
